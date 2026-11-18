//! La couche texte de l'aperçu, dérivée du document Typst compilé.
//!
//! L'aperçu affiche le PDF exporté, mais le texte qui le compose est déjà
//! dans `PagedDocument` : les cadres portent les glyphes, leurs positions,
//! leurs liens et leur source. Cette couche évite de relire le PDF pour
//! sélectionner ou chercher, et reste indépendante du zoom et de la
//! rasterisation.
//!
//! Les coordonnées publiques sont en **points Typst**, avec l'origine en haut
//! à gauche de la page. La vue les convertit en pixels avec le rapport
//! d'affichage de la page.

use std::sync::Arc;

use typst::layout::{Abs, Frame, FrameItem, Point, Rect, Size, Transform};
use typst::model::Destination;
use typst::syntax::Span;
use typst::text::{TextItem, TextItemView};
use typst_layout::PagedDocument;

/// Une sélection de texte dans l'aperçu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionApercu {
    /// Le point de départ du glisser.
    pub ancre: PositionTexte,
    /// Le point courant du glisser.
    pub focus: PositionTexte,
}

/// Une position dans le texte d'une page : le glyphe désigné.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositionTexte {
    /// Page, à partir de zéro.
    pub page: usize,
    /// Rang du glyphe dans l'ordre de lecture de la page.
    pub glyphe: usize,
}

/// Une couche texte complète, partagée par les vues d'un même document.
#[derive(Debug, Clone)]
pub struct DocumentTexte {
    /// Pages dans l'ordre du document.
    pub pages: Vec<PageTexte>,
}

/// Le texte d'une page, ses glyphes et ses liens.
#[derive(Debug, Clone)]
pub struct PageTexte {
    /// Taille de la page, en points.
    pub taille: Size,
    /// Glyphes dans l'ordre de lecture.
    pub glyphes: Vec<GlyphePlace>,
    /// Liens posés sur la page.
    pub liens: Vec<LienPlace>,
    /// Texte de la page, avec les retours de ligne.
    pub texte: String,
    /// Texte normalisé pour la recherche, de même longueur que `texte`.
    pub recherche: String,
    /// Pour chaque caractère de `recherche`, le glyphe qui le porte.
    ///
    /// Les séparateurs insérés (espaces, retours de ligne) n'ont pas de
    /// glyphe : leur entrée vaut `None`.
    pub ancres: Vec<Option<usize>>,
}

impl PageTexte {
    /// Le glyphe le plus proche d'un point de la page.
    ///
    /// Un point dans une boîte gagne tout de suite ; sinon, la distance à la
    /// boîte départage les lignes. C'est ce qui rend un clic dans l'interligne
    /// utile au lieu de ne rien produire.
    pub fn glyphe_au_point(&self, point: Point) -> Option<usize> {
        let mut meilleur: Option<(f64, usize)> = None;

        for (rang, glyphe) in self.glyphes.iter().enumerate() {
            let distance = distance_au_rect(point, glyphe.rect);
            if distance <= 0.0 {
                return Some(rang);
            }

            if meilleur.is_none_or(|(avant, _)| distance < avant) {
                meilleur = Some((distance, rang));
            }
        }

        meilleur.map(|(_, rang)| rang)
    }

    /// Les bornes du mot autour d'un glyphe, dans la même ligne.
    pub fn mot_autour(&self, rang: usize) -> (usize, usize) {
        let Some(glyphe) = self.glyphes.get(rang) else {
            return (rang, rang);
        };
        let ligne = glyphe.ligne;

        let est_mot = |rang: usize| {
            self.glyphes
                .get(rang)
                .is_some_and(|glyphe| glyphe.ligne == ligne && est_caractere_de_mot(&glyphe.texte))
        };

        let mut debut = rang;
        while debut > 0 && est_mot(debut - 1) {
            debut -= 1;
        }

        let mut fin = rang;
        while fin + 1 < self.glyphes.len() && est_mot(fin + 1) {
            fin += 1;
        }

        (debut, fin)
    }

    /// Les bornes d'une ligne autour d'un glyphe.
    pub fn ligne_autour(&self, rang: usize) -> (usize, usize) {
        let Some(glyphe) = self.glyphes.get(rang) else {
            return (rang, rang);
        };
        let ligne = glyphe.ligne;
        let debut = self
            .glyphes
            .iter()
            .position(|glyphe| glyphe.ligne == ligne)
            .unwrap_or(rang);
        let fin = self
            .glyphes
            .iter()
            .rposition(|glyphe| glyphe.ligne == ligne)
            .unwrap_or(rang);

        (debut, fin)
    }
}

/// Un caractère qui appartient à un mot pour le double-clic.
fn est_caractere_de_mot(texte: &str) -> bool {
    texte.chars().next().is_some_and(|caractere| {
        caractere.is_alphanumeric() || caractere == '\'' || caractere == '-'
    })
}

/// Distance d'un point à une boîte ; zéro s'il est dedans.
fn distance_au_rect(point: Point, rect: Rect) -> f64 {
    let dx = if point.x < rect.min.x {
        (rect.min.x - point.x).to_pt()
    } else if point.x > rect.max.x {
        (point.x - rect.max.x).to_pt()
    } else {
        0.0
    };
    let dy = if point.y < rect.min.y {
        (rect.min.y - point.y).to_pt()
    } else if point.y > rect.max.y {
        (point.y - rect.max.y).to_pt()
    } else {
        0.0
    };

    dx.hypot(dy)
}

/// Un glyphe placé sur une page.
#[derive(Debug, Clone)]
pub struct GlyphePlace {
    /// Le texte du glyphe — une ligature peut en porter plusieurs.
    ///
    /// `EcoString` garde les glyphes d'un caractère en ligne : une thèse
    /// entière ne crée pas une allocation par lettre.
    pub texte: typst::ecow::EcoString,
    /// Sa boîte, en points de page.
    pub rect: Rect,
    /// Le rang de sa ligne dans l'ordre de lecture.
    pub ligne: usize,
    /// La source Typst du glyphe.
    pub span: Span,
}

/// Un lien posé sur la page.
#[derive(Debug, Clone)]
pub struct LienPlace {
    /// La zone cliquable, en points de page.
    pub rect: Rect,
    /// La destination du lien.
    pub destination: Destination,
}

impl DocumentTexte {
    /// Extrait la couche texte d'un document compilé.
    ///
    /// Le parcours descend dans les groupes et compose leurs transformations :
    /// un titre placé, une note ou un groupe mis à l'échelle garde ainsi des
    /// positions justes.
    pub fn extraire(document: &PagedDocument) -> Self {
        let pages = document
            .pages()
            .iter()
            .map(|page| {
                let mut page_texte = PageTexte {
                    taille: page.frame.size(),
                    glyphes: Vec::new(),
                    liens: Vec::new(),
                    texte: String::new(),
                    recherche: String::new(),
                    ancres: Vec::new(),
                };

                parcourir(
                    &page.frame,
                    Transform::identity(),
                    &mut page_texte.glyphes,
                    &mut page_texte.liens,
                );
                ordonner(&mut page_texte);
                page_texte
            })
            .collect();

        Self { pages }
    }

    /// Le nombre de pages de la couche.
    pub fn nombre_de_pages(&self) -> usize {
        self.pages.len()
    }

    /// La page demandée, si elle existe.
    pub fn page(&self, index: usize) -> Option<&PageTexte> {
        self.pages.get(index)
    }

    /// Le glyphe le plus proche d'un point, sur la page demandée.
    pub fn position_au_point(&self, page: usize, point: Point) -> Option<PositionTexte> {
        let page_texte = self.page(page)?;
        let glyphe = page_texte.glyphe_au_point(point)?;

        Some(PositionTexte { page, glyphe })
    }

    /// Les glyphes d'une sélection, page par page.
    ///
    /// Une sélection peut traverser plusieurs pages ; l'ordre est celui de
    /// lecture, indépendamment du sens du glisser.
    pub fn selection(&self, ancre: PositionTexte, focus: PositionTexte) -> Vec<GlypheSelection> {
        let (debut, fin) = if ancre <= focus {
            (ancre, focus)
        } else {
            (focus, ancre)
        };

        let mut sortie = Vec::new();

        for page_index in debut.page..=fin.page.min(self.pages.len().saturating_sub(1)) {
            let Some(page) = self.pages.get(page_index) else {
                continue;
            };
            let premier = if page_index == debut.page {
                debut.glyphe
            } else {
                0
            };
            let dernier = if page_index == fin.page {
                fin.glyphe
            } else {
                page.glyphes.len().saturating_sub(1)
            };

            for (rang, glyphe) in page
                .glyphes
                .iter()
                .enumerate()
                .skip(premier)
                .take(dernier.saturating_sub(premier) + 1)
            {
                sortie.push(GlypheSelection {
                    page: page_index,
                    rang,
                    ligne: glyphe.ligne,
                    rect: glyphe.rect,
                    texte: glyphe.texte.clone(),
                });
            }
        }

        sortie
    }

    /// Le texte d'une sélection, dans l'ordre de lecture.
    ///
    /// Les espaces manquants entre deux glyphes d'une même ligne sont
    /// reconstruits ; un changement de ligne ou de page produit un retour à
    /// la ligne.
    pub fn texte_selection(&self, ancre: PositionTexte, focus: PositionTexte) -> String {
        let glyphes = self.selection(ancre, focus);
        let mut sortie = String::new();
        let mut ligne = None;
        let mut page = None;
        let mut precedent: Option<&GlypheSelection> = None;

        for glyphe in &glyphes {
            if page != Some(glyphe.page) {
                if page.is_some() {
                    sortie.push('\n');
                }
                page = Some(glyphe.page);
                ligne = None;
            }

            if ligne != Some(glyphe.ligne) {
                if ligne.is_some() {
                    sortie.push('\n');
                }
                ligne = Some(glyphe.ligne);
            } else if let Some(avant) = precedent
                && !avant.texte.chars().last().is_some_and(char::is_whitespace)
                && !glyphe.texte.chars().next().is_some_and(char::is_whitespace)
                && glyphe.rect.min.x - avant.rect.max.x > Abs::pt(0.35)
            {
                sortie.push(' ');
            }

            sortie.push_str(&glyphe.texte);
            precedent = Some(glyphe);
        }

        sortie
    }

    /// Les rectangles à surligner pour une sélection, une page à la fois.
    pub fn surlignage_selection(
        &self,
        page: usize,
        ancre: PositionTexte,
        focus: PositionTexte,
    ) -> Vec<Rect> {
        let glyphes: Vec<Rect> = self
            .selection(ancre, focus)
            .into_iter()
            .filter(|glyphe| glyphe.page == page)
            .map(|glyphe| glyphe.rect)
            .collect();

        fusionner_par_ligne(&glyphes, self.page(page))
    }

    /// Les rectangles à surligner pour un résultat de recherche.
    pub fn surlignage_resultat(&self, page: usize, debut: usize, fin: usize) -> Vec<Rect> {
        let Some(page_texte) = self.page(page) else {
            return Vec::new();
        };
        let rects: Vec<Rect> = page_texte
            .glyphes
            .iter()
            .enumerate()
            .filter(|(rang, _)| *rang >= debut && *rang <= fin)
            .map(|(_, glyphe)| glyphe.rect)
            .collect();

        fusionner_par_ligne(&rects, Some(page_texte))
    }
}

/// Un glyphe retenu par une sélection.
#[derive(Debug, Clone)]
pub struct GlypheSelection {
    pub page: usize,
    pub rang: usize,
    pub ligne: usize,
    pub rect: Rect,
    pub texte: typst::ecow::EcoString,
}

/// Fusionne les boîtes d'une même ligne pour ne dessiner qu'un rectangle.
fn fusionner_par_ligne(rects: &[Rect], page: Option<&PageTexte>) -> Vec<Rect> {
    let Some(page) = page else {
        return Vec::new();
    };

    let mut sortie = Vec::new();
    let mut courant: Option<(usize, Rect)> = None;

    for rect in rects {
        let ligne = page
            .glyphes
            .iter()
            .find(|glyphe| glyphe.rect == *rect)
            .map_or(0, |glyphe| glyphe.ligne);

        match &mut courant {
            Some((ligne_courante, boite)) if *ligne_courante == ligne => {
                boite.min = boite.min.min(rect.min);
                boite.max = boite.max.max(rect.max);
            }
            Some((_, boite)) => {
                sortie.push(*boite);
                courant = Some((ligne, *rect));
            }
            None => courant = Some((ligne, *rect)),
        }
    }

    if let Some((_, boite)) = courant {
        sortie.push(boite);
    }

    sortie
}

/// Parcourt un cadre, en composant les transformations des groupes.
fn parcourir(
    frame: &Frame,
    transform: Transform,
    glyphes: &mut Vec<GlyphePlace>,
    liens: &mut Vec<LienPlace>,
) {
    for (position, item) in frame.items() {
        match item {
            FrameItem::Text(texte) => {
                extraire_texte(texte, *position, transform, glyphes);
            }
            FrameItem::Group(groupe) => {
                let enfant = transform.pre_concat(
                    Transform::translate(position.x, position.y).pre_concat(groupe.transform),
                );
                parcourir(&groupe.frame, enfant, glyphes, liens);
            }
            FrameItem::Link(destination, taille) => {
                let boite = Rect::from_pos_size(*position, *taille);
                liens.push(LienPlace {
                    rect: transformer_rect(boite, transform),
                    destination: destination.clone(),
                });
            }
            _ => {}
        }
    }
}

/// Extrait les glyphes d'un `TextItem`.
fn extraire_texte(
    texte: &TextItem,
    position: Point,
    transform: Transform,
    sortie: &mut Vec<GlyphePlace>,
) {
    let vue = TextItemView::full(texte);
    let mut x = Abs::zero();
    let mut y = Abs::zero();
    // `position` reste dans le repère du cadre enfant : la transformation du
    // groupe est appliquée **une seule fois**, à la boîte finale. La composer
    // aussi sur l'origine la ferait subir deux fois — un groupe translaté
    // décalait alors tout son texte vers le bas et la droite.
    let ascender = texte.font.metrics().ascender.at(texte.size);
    let descender = texte.font.metrics().descender.at(texte.size);

    for glyphe in &texte.glyphs {
        let texte_glyphe = vue.glyph_text(glyphe);
        let x_offset = x + glyphe.x_offset.at(texte.size);
        let baseline = position.y + y - glyphe.y_offset.at(texte.size);
        let x0 = position.x + x_offset;
        let x1 = x0 + glyphe.x_advance.at(texte.size).max(Abs::pt(0.1));
        let boite = Rect::new(
            Point::new(x0, baseline - ascender),
            Point::new(x1, baseline - descender),
        );

        sortie.push(GlyphePlace {
            texte: texte_glyphe,
            rect: transformer_rect(boite, transform),
            ligne: 0,
            span: glyphe.span.0,
        });

        x += glyphe.x_advance.at(texte.size);
        y += glyphe.y_advance.at(texte.size);
    }
}

/// Transforme une boîte et renvoie sa boîte englobante.
fn transformer_rect(rect: Rect, transform: Transform) -> Rect {
    let a = rect.min.transform(transform);
    let b = Point::new(rect.max.x, rect.min.y).transform(transform);
    let c = Point::new(rect.min.x, rect.max.y).transform(transform);
    let d = rect.max.transform(transform);

    let min = a.min(b).min(c).min(d);
    let max = a.max(b).max(c).max(d);
    Rect::new(min, max)
}

/// Ordonne les glyphes en lignes, puis construit le texte de page.
fn ordonner(page: &mut PageTexte) {
    page.glyphes.sort_by(|a, b| {
        let ya = a.rect.min.y.to_pt();
        let yb = b.rect.min.y.to_pt();
        ya.total_cmp(&yb)
            .then_with(|| a.rect.min.x.to_pt().total_cmp(&b.rect.min.x.to_pt()))
    });

    let mut ligne = 0;
    let mut ligne_courante: Option<f64> = None;
    let mut dernier_x: Option<f64> = None;
    let mut texte = String::new();
    let mut recherche = String::new();
    let mut ancres = Vec::new();

    for (rang, glyphe) in page.glyphes.iter_mut().enumerate() {
        let y = glyphe.rect.min.y.to_pt();
        match ligne_courante {
            Some(avant) if (y - avant).abs() <= 2.0 => {}
            Some(_) => {
                ligne += 1;
                texte.push('\n');
                recherche.push(' ');
                ancres.push(None);
                dernier_x = None;
            }
            None => {}
        }
        ligne_courante = Some(y);
        glyphe.ligne = ligne;

        if let Some(avant) = dernier_x {
            let ecart = glyphe.rect.min.x.to_pt() - avant;
            if ecart > 0.35
                && !texte.ends_with(char::is_whitespace)
                && !glyphe.texte.starts_with(char::is_whitespace)
            {
                texte.push(' ');
                recherche.push(' ');
                ancres.push(None);
            }
        }

        for caractere in glyphe.texte.chars() {
            texte.push(caractere);
            recherche.push(plier(caractere));
            ancres.push(Some(rang));
        }

        dernier_x = Some(glyphe.rect.max.x.to_pt());
    }

    page.texte = texte;
    page.recherche = recherche;
    page.ancres = ancres;
}

/// Plie un caractère pour la recherche : casse et accents français.
pub(crate) fn plier(caractere: char) -> char {
    match caractere {
        'A'..='Z' => caractere.to_ascii_lowercase(),
        'À' | 'Á' | 'Â' | 'Ä' | 'Ã' | 'Å' => 'a',
        'Ç' => 'c',
        'È' | 'É' | 'Ê' | 'Ë' => 'e',
        'Ì' | 'Í' | 'Î' | 'Ï' => 'i',
        'Ñ' => 'n',
        'Ò' | 'Ó' | 'Ô' | 'Ö' | 'Õ' => 'o',
        'Ù' | 'Ú' | 'Û' | 'Ü' => 'u',
        'Ý' | 'Ÿ' => 'y',
        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        'Œ' => 'œ',
        'œ' => 'œ',
        _ => caractere.to_lowercase().next().unwrap_or(caractere),
    }
}

/// Le document texte d'une compilation, prêt à partager.
pub fn extraire(document: &PagedDocument) -> Arc<DocumentTexte> {
    Arc::new(DocumentTexte::extraire(document))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn compiler(source: &str) -> PagedDocument {
        let dir = std::env::temp_dir().join(format!(
            "typst-writer-texte-{}-{}",
            std::process::id(),
            source.len()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier de test");
        std::fs::write(dir.join("main.typ"), source).expect("source");
        let outcome = crate::typst::render::compile(&dir, source, &[]);
        assert!(
            outcome.errors.is_empty(),
            "compilation : {:?}",
            outcome.errors
        );
        outcome.document.expect("document compilé").as_ref().clone()
    }

    fn page(document: &DocumentTexte, index: usize) -> &PageTexte {
        document.page(index).expect("page")
    }

    #[test]
    fn le_texte_d_un_groupe_est_transforme_une_seule_fois() {
        let document = compiler("#rotate(10deg)[Bonjour]\n");
        let texte = DocumentTexte::extraire(&document);
        let page = page(&texte, 0);
        let glyphe = page.glyphes.first().expect("un glyphe");

        // Le groupe porte la marge de page et la rotation. Une origine
        // transformée deux fois envoyait ce texte plus bas et plus à droite ;
        // la boîte doit rester dans la page et autour de la marge.
        assert!(
            glyphe.rect.min.x.to_pt() > 60.0 && glyphe.rect.min.x.to_pt() < 80.0,
            "x du premier glyphe : {:?}",
            glyphe.rect
        );
        assert!(
            glyphe.rect.min.y.to_pt() > 55.0 && glyphe.rect.min.y.to_pt() < 80.0,
            "y du premier glyphe : {:?}",
            glyphe.rect
        );
        assert!(glyphe.rect.max.x.to_pt() <= page.taille.x.to_pt());
        assert!(glyphe.rect.max.y.to_pt() <= page.taille.y.to_pt());
    }

    #[test]
    fn extrait_le_texte_les_glyphes_et_les_liens() {
        let document = compiler(
            "= Un titre\n\nBonjour le monde.\n\n#link(\"https://example.org\")[Un lien]\n",
        );
        let texte = DocumentTexte::extraire(&document);
        let page = page(&texte, 0);

        assert!(
            page.texte.contains("Bonjour le monde"),
            "texte : {}",
            page.texte
        );
        assert!(page.texte.contains("Un titre"), "titre : {}", page.texte);
        assert!(page.glyphes.len() > 10);
        assert!(
            page.liens
                .iter()
                .any(|lien| matches!(lien.destination, Destination::Url(_))),
            "le lien externe est conservé"
        );
        assert!(page.recherche.contains("bonjour le monde"));
        assert_eq!(page.recherche.chars().count(), page.ancres.len());
    }

    #[test]
    fn le_lien_interne_est_conserve() {
        let document = compiler("= Un titre <titre>\n\nVoir #link(<titre>)[le titre].\n");
        let texte = DocumentTexte::extraire(&document);
        let page = page(&texte, 0);

        assert!(
            page.liens
                .iter()
                .any(|lien| matches!(lien.destination, Destination::Location(_))),
            "le lien interne est conservé : {:?}",
            page.liens
        );
    }

    #[test]
    fn les_glyphes_sont_ordonnes_et_leurs_boites_sont_dans_la_page() {
        let document = compiler("Bonjour le monde.\n\nDeuxième ligne.\n");
        let texte = DocumentTexte::extraire(&document);
        let page = page(&texte, 0);

        assert!(page.glyphes.windows(2).all(|paire| {
            let a = &paire[0];
            let b = &paire[1];
            a.ligne < b.ligne || (a.ligne == b.ligne && a.rect.min.x <= b.rect.min.x)
        }));

        let largeur = page.taille.x.to_pt();
        let hauteur = page.taille.y.to_pt();

        for glyphe in &page.glyphes {
            assert!(glyphe.rect.min.x.to_pt() >= -0.5);
            assert!(glyphe.rect.min.y.to_pt() >= -0.5);
            assert!(glyphe.rect.max.x.to_pt() <= largeur + 1.0);
            assert!(glyphe.rect.max.y.to_pt() <= hauteur + 1.0);
        }
    }

    #[test]
    fn le_texte_d_une_selection_reconstruit_les_espaces_et_les_lignes() {
        let document = compiler("Bonjour le monde.\n\nUne autre ligne.\n");
        let texte = DocumentTexte::extraire(&document);
        let page = page(&texte, 0);
        let premier = PositionTexte { page: 0, glyphe: 0 };
        let dernier = PositionTexte {
            page: 0,
            glyphe: page.glyphes.len() - 1,
        };

        let selection = texte.texte_selection(premier, dernier);

        assert!(
            selection.contains("Bonjour le monde"),
            "sélection : {selection:?}"
        );
        assert!(selection.contains('\n'));
        assert!(!selection.is_empty());
    }

    #[test]
    fn une_selection_inversee_lit_dans_le_bon_sens() {
        let document = compiler("Bonjour le monde.\n");
        let texte = DocumentTexte::extraire(&document);
        let debut = PositionTexte { page: 0, glyphe: 3 };
        let fin = PositionTexte { page: 0, glyphe: 9 };

        assert_eq!(
            texte.texte_selection(fin, debut),
            texte.texte_selection(debut, fin)
        );
    }

    #[test]
    fn la_recherche_plie_les_accents_et_la_casse() {
        assert_eq!(plier('É'), 'e');
        assert_eq!(plier('é'), 'e');
        assert_eq!(plier('Ç'), 'c');
        assert_eq!(plier('A'), 'a');
        assert_eq!(plier('ß'), 'ß');
    }

    #[test]
    fn le_point_choisit_le_glyphe_le_plus_proche() {
        let page = PageTexte {
            taille: Size::new(Abs::pt(500.0), Abs::pt(700.0)),
            glyphes: vec![
                GlyphePlace {
                    texte: "A".into(),
                    rect: Rect::new(
                        Point::new(Abs::pt(10.0), Abs::pt(10.0)),
                        Point::new(Abs::pt(20.0), Abs::pt(24.0)),
                    ),
                    ligne: 0,
                    span: Span::detached(),
                },
                GlyphePlace {
                    texte: "B".into(),
                    rect: Rect::new(
                        Point::new(Abs::pt(24.0), Abs::pt(10.0)),
                        Point::new(Abs::pt(34.0), Abs::pt(24.0)),
                    ),
                    ligne: 0,
                    span: Span::detached(),
                },
            ],
            liens: Vec::new(),
            texte: "AB".to_string(),
            recherche: "ab".to_string(),
            ancres: vec![Some(0), Some(1)],
        };

        assert_eq!(
            page.glyphe_au_point(Point::new(Abs::pt(15.0), Abs::pt(15.0))),
            Some(0)
        );
        assert_eq!(
            page.glyphe_au_point(Point::new(Abs::pt(30.0), Abs::pt(15.0))),
            Some(1)
        );
        assert_eq!(
            page.glyphe_au_point(Point::new(Abs::pt(15.0), Abs::pt(40.0))),
            Some(0)
        );
    }

    #[test]
    fn le_double_clic_prend_le_mot_et_le_triple_la_ligne() {
        let page = PageTexte {
            taille: Size::new(Abs::pt(500.0), Abs::pt(700.0)),
            glyphes: vec![
                GlyphePlace {
                    texte: "Bonjour".into(),
                    rect: Rect::new(
                        Point::new(Abs::pt(10.0), Abs::pt(10.0)),
                        Point::new(Abs::pt(60.0), Abs::pt(24.0)),
                    ),
                    ligne: 0,
                    span: Span::detached(),
                },
                GlyphePlace {
                    texte: " ".into(),
                    rect: Rect::new(
                        Point::new(Abs::pt(60.0), Abs::pt(10.0)),
                        Point::new(Abs::pt(64.0), Abs::pt(24.0)),
                    ),
                    ligne: 0,
                    span: Span::detached(),
                },
                GlyphePlace {
                    texte: "monde".into(),
                    rect: Rect::new(
                        Point::new(Abs::pt(64.0), Abs::pt(10.0)),
                        Point::new(Abs::pt(104.0), Abs::pt(24.0)),
                    ),
                    ligne: 0,
                    span: Span::detached(),
                },
                GlyphePlace {
                    texte: "Suite".into(),
                    rect: Rect::new(
                        Point::new(Abs::pt(10.0), Abs::pt(40.0)),
                        Point::new(Abs::pt(50.0), Abs::pt(54.0)),
                    ),
                    ligne: 1,
                    span: Span::detached(),
                },
            ],
            liens: Vec::new(),
            texte: "Bonjour monde\nSuite".to_string(),
            recherche: "bonjour monde suite".to_string(),
            ancres: vec![
                Some(0),
                Some(0),
                Some(0),
                Some(0),
                Some(0),
                Some(0),
                Some(0),
                None,
                Some(2),
                Some(2),
                Some(2),
                Some(2),
                Some(2),
                None,
                Some(3),
                Some(3),
                Some(3),
                Some(3),
                Some(3),
            ],
        };

        assert_eq!(page.mot_autour(2), (2, 2));
        assert_eq!(page.mot_autour(0), (0, 0));
        assert_eq!(page.ligne_autour(2), (0, 2));
    }

    #[test]
    fn le_texte_de_page_est_independant_du_chemin() {
        // Le test ne doit pas dépendre d'un projet utilisateur.
        let _ = Path::new(".");
        let document = compiler("Un texte simple.\n");
        let texte = DocumentTexte::extraire(&document);

        assert!(texte.nombre_de_pages() >= 1);
    }
}
