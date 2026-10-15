//! Le journal de mesure : la mémoire résidente et la mémoire graphique.
//!
//! Un instrument, pas une fonction de l'application : rien ici ne tourne tant
//! que `TYPST_WRITER_MEMOIRE` n'est pas posé. Il existe parce qu'un soupçon
//! d'explosion mémoire ne se diagnostique pas à l'œil, et parce que la mémoire
//! graphique n'apparaît dans aucun profil ordinaire.

use super::{MEMORY_INTERVAL_S, Message, State};

use iced::{Subscription, time};

/// Suit l'empreinte mémoire, si l'environnement le demande.
///
/// Désactivé par défaut : c'est un instrument de mesure, pas une fonction de
/// l'application. Il existe parce qu'un rapport d'explosion mémoire ne se
/// diagnostique pas à l'œil — trois campagnes de mesure n'ont pas réussi à le
/// reproduire — et que la mémoire graphique n'apparaît dans aucun profil
/// ordinaire.
///
///     TYPST_WRITER_MEMOIRE=1 cargo run --release
pub(super) fn subscription_memory() -> Subscription<Message> {
    if !crate::app::epreuves::actif("memoire") {
        return Subscription::none();
    }

    time::every(std::time::Duration::from_secs(MEMORY_INTERVAL_S)).map(|_| Message::MemoryTick)
}

/// Un relevé de mémoire.
#[derive(Debug, Clone, Copy)]
pub(super) struct Memory {
    /// Mémoire résidente de l'application, en kilo-octets.
    resident: usize,

    /// Mémoire graphique « GTT » du système, en kilo-octets.
    gtt: usize,

    /// Mémoire graphique « VRAM » du système, en kilo-octets.
    vram: usize,
}

/// Relève et journalise l'empreinte mémoire.
///
/// Trois chiffres, parce qu'aucun ne suffit : la mémoire résidente de
/// l'application, et celles du sous-système graphique. Ces dernières sont
/// **celles de tout le système** — le compositeur et les autres applications y
/// figurent aussi — et seuls leurs **écarts** depuis le premier relevé ont un
/// sens : c'est la dérive qu'on cherche, pas la valeur absolue.
pub(super) fn report_memory(state: &State) {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    static BASELINE: std::sync::OnceLock<Memory> = std::sync::OnceLock::new();

    let now = Memory {
        resident: resident_memory_kb(),
        gtt: graphics_memory_kb("mem_info_gtt_used"),
        vram: graphics_memory_kb("mem_info_vram_used"),
    };

    let start = *START.get_or_init(std::time::Instant::now);
    let baseline = *BASELINE.get_or_init(|| now);

    let delta = |now: usize, baseline: usize| (now as i64 - baseline as i64) / 1024;

    let pages = state
        .render
        .document
        .as_deref()
        .map_or(0, crate::typst::render::page_count);

    // La densité réellement employée pour une page : c'est elle qui règle la
    // mémoire, et c'est en la lisant d'une version à l'autre qu'on voit ce
    // qu'une optimisation a changé.
    let density = match state.render.document.as_deref() {
        Some(document) => crate::typst::render::pixel_per_pt(
            crate::ui::preview::display_ratio(
                state
                    .render
                    .viewport
                    .unwrap_or_else(crate::ui::preview::default_viewport),
                crate::typst::render::page_size(document, 0).unwrap_or((595.3, 841.9)),
            ),
            state.scale_factor,
        ),
        None => 0.0,
    };

    let rasterizations = rasterizations_since_last_report(state.render.requested);

    println!(
        "[mémoire] {:>4} s | RSS {} Mo ({:+}) | GTT système {} Mio ({:+}) | VRAM système {} Mio ({:+}) \
         | pages {pages} (en cache {}, en cours {}) | rendu à {density:.2} px/pt \
         | {rasterizations} rasterisation(s) depuis le dernier relevé",
        start.elapsed().as_secs(),
        now.resident / 1024,
        delta(now.resident, baseline.resident),
        now.gtt / 1024,
        delta(now.gtt, baseline.gtt),
        now.vram / 1024,
        delta(now.vram, baseline.vram),
        state.render.page_cache.len(),
        state.render.rendering.len(),
    );
}

/// Nombre de rasterisations demandées depuis le relevé précédent.
///
/// C'est la **cadence** qui compte, et non le total : au repos l'aperçu n'en
/// demande aucune, un défilement quelques-unes, et un cycle en demande des
/// dizaines par seconde — plusieurs mégaoctets à chaque tour. Sans ce chiffre,
/// une boucle de rasterisation est invisible : la mémoire monte, et rien ne dit
/// pourquoi.
pub(super) fn rasterizations_since_last_report(requested: u64) -> u64 {
    static PREVIOUS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    let previous = PREVIOUS.swap(requested, std::sync::atomic::Ordering::Relaxed);

    requested.saturating_sub(previous)
}

/// Relève l'état de l'assistance, quand l'environnement demande le journal.
///
/// Le même instrument que la mémoire, et pour la même raison : un serveur de
/// langage est un sous-système qu'on ne voit pas. Sans ce relevé, « l'assistance
/// est branchée » ne se vérifie qu'à l'œil, sur une barre d'état.
pub(super) fn report_lsp(state: &State) {
    if state.lsp.session.is_none() && !state.lsp.unavailable {
        return;
    }

    let (errors, total) = crate::lsp::messages::counts(&state.lsp.diagnostics);

    println!(
        "[assistance] {} | document {} | {errors} erreur(s) sur {total} diagnostic(s) | {}",
        if state.lsp.unavailable {
            "absente"
        } else if state.lsp.ready {
            "active"
        } else {
            "au démarrage"
        },
        if state.lsp.document.is_some() {
            "ouvert"
        } else {
            "aucun"
        },
        if state.lsp.awaiting {
            "réponse attendue"
        } else {
            "au repos"
        },
    );

    if !verbose() {
        return;
    }

    // Ce que fait la liste — parce que « un menu jamais ouvert » et « un menu
    // ouvert hors de l'écran » se ressemblent exactement, et qu'aucun des deux ne
    // se voit dans une capture d'écran.
    let editor = &state.editor.editor;
    let document_y = editor
        .cursor_screen_position()
        .map_or(f32::NAN, |point| point.y);

    match state.lsp.completion.as_ref() {
        None => println!("[assistance] liste : fermée"),
        Some(open) => println!(
            "[assistance] liste : ouverte | déclencheur `{}` | mot « {} » | {} élément(s), {} montré(s), sélection {} | ancre ({:.0}, {:.0}) | curseur dans le document : {:.0} | défilement {:.0}, vue {:.0}",
            open.trigger,
            open.typed,
            open.items.len(),
            open.shown.len(),
            open.selected,
            open.anchor.x,
            open.anchor.y,
            document_y,
            editor.viewport_scroll(),
            editor.viewport_height(),
        ),
    }
}

/// Le journal détaillé est-il demandé ?
fn verbose() -> bool {
    crate::app::epreuves::actif("lsp")
}

/// Mémoire résidente du processus, en kilo-octets.
pub(super) fn resident_memory_kb() -> usize {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmRSS:")?
                    .trim()
                    .trim_end_matches(" kB")
                    .trim()
                    .parse()
                    .ok()
            })
        })
        .unwrap_or(0)
}

/// Somme d'un compteur de mémoire graphique, en kilo-octets.
///
/// Ces compteurs appartiennent au noyau (`/sys/class/drm`) et couvrent **tout le
/// système**. Ils valent zéro là où ils n'existent pas, ce qui est le cas de
/// toute autre plateforme que Linux — la mesure ne doit jamais gêner.
pub(super) fn graphics_memory_kb(counter: &str) -> usize {
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return 0;
    };

    let bytes: usize = entries
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("card"))
        .filter_map(|entry| std::fs::read_to_string(entry.path().join("device").join(counter)).ok())
        .filter_map(|value| value.trim().parse::<usize>().ok())
        .sum();

    bytes / 1024
}
