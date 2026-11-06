# Proposition d'architecture — un lecteur PDF dans la fenêtre de rendu

*Étude préparatoire à bug-report-9, tâche 4 (« Refonte du moteur de
rendu »). Aucune implémentation n'est engagée : ce document est la
proposition à examiner avant tout code.*

## 1. Ce que le code fait aujourd'hui (rectification du rapport)

Le rapport 4.1 dit : « l'application semble récupérer les vecteurs
produits par le moteur de composition Typst, puis les dessiner elle-même ».
Le code dit autre chose (`src/typst/render.rs`) :

1. **Compilation** — Typst est appelé en bibliothèque (`compile`) : le
   document complet est composé en ~15 ms, sans aucun bitmap. Les
   diagnostics et l'introspection (titres → pages) en sortent aussi.
2. **Rendu raster** — chaque page visible est rasterisée en pixels RGBA
   par `typst-render 0.15.1` (le moteur d'export du CLI Typst, qui
   dessine les glyphes directement). Rien n'est « dessiné à la main ».
3. **Export PDF** — `render_pdf` produit déjà le vrai PDF (l'export
   « Exporter en PDF »).

Le **zoom pixellise** pour une raison précise et documentée dans le
code : `MAX_PIXEL_PER_PT = 2.0` borne la densité de rasterisation
(une page A4 à 2 px/pt pèse ~8 Mo de pixels ; la borne est là pour la
mémoire). Quand le zoom dépasse cette densité, l'image est **étirée** —
c'est la pixellisation observée. Le correctif de qualité n'est donc pas
« avoir des vecteurs » : aucun écran n'affiche des vecteurs ; c'est
**rasteriser à la densité exacte du zoom, par tuiles, sur la seule
région visible** — quel que soit le moteur de rendu.

Cela change la nature de la question 4 : la bonne nouvelle est que
« afficher le PDF dans la fenêtre de rendu » est un vrai projet, mais
son bénéfice est ailleurs que dans la netteté (voir § 4).

## 2. Étude des bibliothèques (Rust, compatibles Iced / Typst)

| Bibliothèque | Licence | Nature | Verdict |
|---|---|---|---|
| **pdfium-render 0.9** (+ pdfium-binaries) | BSD-3-Clause (PDFium, projet Chromium) | Liaisons Rust vers PDFium, liées à l'exécution ; rendu bitmap à toute densité, extraction de texte, liens, formulaires ; binaires précompilés Linux/Windows/macOS | **Retenu** |
| mupdf-rs 0.8 (`mupdf` / `mupdf-sys`) | **AGPL-3.0** | Liaisons MuPDF, très complet (rendu, SVG, édition) | **Écarté** : incompatible avec la licence MIT du projet, à moins de basculer tout le projet en AGPL (décision de l'auteur, pas technique) |
| poppler-rs | **GPL-2** (« every program linking against this crate must be GPL ») | Liaisons poppler-glib/cairo, testé Linux | **Écarté** : GPL + dépendances glib/cairo, mal embarquables sur Windows/macOS |
| `pdf_render` 1.3 (PDFluent) | Apache-2.0 OU MIT | Rasteriseur PDF **pur Rust** (vello_cpu), sans C++, comparable au référentiel par SSIM | **Piste sérieuse à terme** — mais jeune, limites documentées (PDF chiffrés, blending/isolation, knockout) ; Typst produit des PDF simples, donc les limites toucheraient peu nos documents |
| `hayro` | MIT/Apache | Interpréteur et rendu pur Rust | **Écarté pour l'instant** : expérimental, performances non travaillées (déclaré par ses auteurs) |
| `lopdf`, `pdf` (pdf-rs) | MIT/Apache | Lecture/écriture de structures PDF, **sans rendu** | **Écarté** : réimplémenter un rendu contredit la tâche 4.3 (« privilégier un lecteur PDF existant ») |
| (zéro dépendance : garder `typst-render`) | MIT (Typst) | Rasterisation Typst directe, déjà en place | **Alternative** décrite au § 5 |

Contrainte de licence du projet : `license = "MIT"` (Cargo.toml). Seuls
PDFium (BSD-3) et les rasteriseurs purs Rust (MIT/Apache) restent
compatibles sans décision de relicenciement.

## 3. Architecture proposée

Un seul principe : **le PDF devient la source d'affichage, sans toucher
à ce qui l'entoure**. Le langage, le pipeline, l'export, les normes, la
bibliographie et le glossaire ne bougent pas.

```
  document.tw ──▶ pipeline (inchangé) ──▶ Typst compile ──▶ PagedDocument
                                                   │
                                   ┌───────────────┴───────────────┐
                                   ▼                               ▼
                       export PDF (render_pdf, existant)   introspection (titres/pages)
                                   │
                                   ▼
                     ┌──────────────────────────┐
                     │  lecteur::MoteurPdf      │  (nouveau, src/lecteur/)
                     │  ├─ Pdfium  (impl 1)     │  pdfium-render + binaire BSD-3
                     │  └─ TypstRender (impl 2) │  l'existant, derrière le même trait
                     └──────────────────────────┘
                                   │  page → pixels RGBA (PageImage, type actuel)
                                   ▼
                 aperçu iced (widget inchangé, cache double tampon existant)
```

- **`src/lecteur/` (nouveau module)** : un trait `MoteurPdf`
  (`ouvrir(bytes)`, `pages()`, `rendre(page, densite, region)`), deux
  implémentations : `Pdfium` (le lecteur) et `TypstRender` (le chemin
  actuel, repli et témoin de non-régression). La fenêtre de rendu ne
  connaît que le trait : la bascule est un réglage de construction, pas
  une refonte.
- **La compilation reste unique** : un `compile` produit le
  `PagedDocument` (diagnostics, introspection, aperçu instantané) **et**
  les octets du PDF (`render_pdf`, déjà écrit). Le lecteur n'ouvre que
  ces octets — zéro travail de composition en double.
- **Zoom de qualité (4.2)** : au lieu d'étirer une page plafonnée à
  2 px/pt, la fenêtre visible se rasterise **à la densité du zoom**, par
  **tuiles** quand la page entière dépasserait la limite mémoire (une
  tuile de 1024² px à la densité exacte, rendue à la demande). C'est le
  même mécanisme de « pages visibles déjà en vol » qu'aujourd'hui,
  affiné au niveau de la région. La netteté est la même pour les deux
  implémentations du trait — c'est la stratégie de rendu, pas la
  bibliothèque, qui donne la qualité.
- **Fonctions du lecteur (4.3)** : navigation (le suivi titres → pages
  est déjà éprouvé par l'introspection Typst, il reste acquis), liens
  internes et externes, sélection et recherche de texte (APIs texte de
  PDFium), et la garantie que **ce qu'on montre est exactement le PDF
  exporté** — le WYSIWYG cesse d'être une promesse pour devenir une
  identité de fichier.
- **Iced, sans surprise** : la sortie du trait est le `PageImage`
  d'aujourd'hui ; le widget, le cache par images, la virtualisation du
  viewport et les commandes de zoom (barre supérieure, bug-report-9
  T2.1) ne changent pas.

## 4. Ce que la refonte apporte, et ce qu'elle n'apporte pas

Apporte : l'identité aperçu = export ; la sélection/recherche de texte ;
les liens cliquables ; la fidélité aux particularités PDF (couleurs,
polices intégrées à l'export) ; un socle pour annoter plus tard.

N'apporte pas : la netteté du zoom (affaire de stratégie de
rasterisation, § 1) ; la vitesse du premier aperçu (la compilation
Typst reste le premier pas, ~15 ms) ; rien pour l'éditeur.

C'est pourquoi l'étude recommande la **stratégie de tuiles** quel que
soit le moteur — et propose le lecteur pour ce qu'il sait faire de
mieux : être un lecteur.

## 5. Alternative sans nouvelle dépendance (à comparer honnêtement)

Garder `typst-render` seul et corriger la pixellisation à la racine :
rendre par tuiles à la densité du zoom, sans plafond étiré. Coût : une
bibliothèque de moins, aucun empaquetage de binaire ; perte : tout le
§ 4 sauf la netteté. C'est l'option **si** l'aperçu doit rester léger
en dépendances — elle est prévue comme implémentation n°2 du trait
`MoteurPdf`, donc les deux options restent ouvertes **jusqu'au dernier
moment** : commencer par elle (récompense immédiate sur la netteté),
puis ajouter PDFium derrière le trait.

## 6. Étapes d'implémentation (proposées, non engagées)

1. **E1 — la stratégie de tuiles** dans le chemin actuel (netteté du
   zoom, indépendante du lecteur) ; preuve : une épreuve qui rasterise
   une page à 4 px/pt par tuiles et compare un fragment au rendu plein.
2. **E2 — le trait `MoteurPdf`** et son implémentation `TypstRender` ;
   preuve : la suite existante passe derrière le trait, pixel-identique.
3. **E3 — PDFium derrière le trait** (feature `lecteur-pdf`) :
   empaquetage des binaires BSD-3 dans les paquets de release, notice
   d'attribution ; preuve : le texte rendu par le lecteur == le texte de
   `pdftotext` sur les documents synthétiques (les courses d'acceptation
   existantes rejouées sur le lecteur).
4. **E4 — fonctions lecteur** : liens cliquables, sélection, recherche.
5. **E5 — retrait** de l'ancien chemin le jour où le lecteur est jugé.

## 7. Risques et points de vigilance

- **Empaquetage** : ~10-20 Mo par plateforme pour le binaire PDFium
  (dynamic linking, `pdfium-binaries`) ; la licence BSD-3 exige
  l'attribution — dans les paquets et le README. Cross-compilation
  Windows/macOS : les binaires sont fournis en amont, la CI ne les
  compile pas.
- **Premier rendu** : l'ouverture du document PDF par PDFium est
  quasi instantanée ; c'est la compilation Typst qui ouvre le bal.
- **Mémoire à fort zoom** : les tuiles bornent la mémoire, mais le
  nombre de tuiles en vol doit rester plafonné (reprendre la règle des
  « pages visibles non déjà en vol »).
- **fidélité** : PDFium rend ce que Typst exporte — si l'export a un
  défaut, il devient visible à l'écran (c'est voulu : l'aperçu cesse de
  masquer les défauts du PDF livré).
- **`pdf_render` (pur Rust)** : à réévaluer à E5 ; si ses limites
  (chiffrement, blending) ne touchent pas nos documents, il supprime
  tout empaquetage de binaire C++.

## Sources consultées (2026-11-05)

- docs.rs — `pdfium-render 0.9.4` (interface, liaisons à l'exécution)
- github.com/bblanchon/pdfium-binaries (binaires Linux/Windows/macOS)
- docs.rs — `mupdf 0.8.0` et README mupdf-rs (« released under the
  AGPL-3.0 license »)
- docs.rs / github.com/mbr/poppler-rs (« every program or library
  linking against this crate must be GPL licensed as well »)
- crates.io — `pdf-render 1.3.0` (« Apache-2.0 OR MIT », vello_cpu)
- github.com/svammxiahilly/hayro (« experimental, work-in-progress »,
  performance « has not been a focus »)
- `src/typst/render.rs` (compilation, `MAX_PIXEL_PER_PT`, `render_pdf`)
