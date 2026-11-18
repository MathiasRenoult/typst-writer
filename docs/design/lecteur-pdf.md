# Lecteur PDF — recherche et recommandations

> **Statut : implémenté le 16 novembre 2026.**
> La couche texte est extraite de `PagedDocument` ; sélection, copie,
> recherche, liens et priorité de rasterisation sont en place. La vue de
> rendu réexporte son PDF depuis le document déjà compilé, sans relancer
> Typst. Ce document garde la comparaison des solutions et le découpage
> d'origine ; les écarts et les suites différées sont consignés dans
> `agent/travaux-restructuration.md`.
>
> Objet : rendre l'aperçu PDF plus fluide et lui ajouter la sélection de
> texte, la recherche, les liens et les gestes de navigation attendus d'un
> lecteur moderne, sans mettre en danger la fidélité au PDF exporté.

## 1. Ce que fait le lecteur aujourd'hui

L'aperçu est déjà un lecteur sérieux, mais c'est un **lecteur d'images** :

- `PagedDocument` est le document compilé par Typst ; il porte les pages,
  les cadres, les glyphes, les liens et l'introspection
  (`RenderState.document`, `src/app/state.rs:1026-1032`).
- Le PDF exporté est ouvert par `pdf-render` et devient la source de
  l'affichage (`RenderState.lecteur`, même endroit ; `src/typst/render.rs:430-523`).
- Chaque page est rasterisée par région visible, à la densité de l'écran,
  puis affichée comme une image (`src/app/rasterisation.rs:41-103`,
  `src/ui/preview.rs:495-561`). Le cache est borné, les textures sont
  stables, le double tampon évite les cadres vides : la fluidité a déjà été
  travaillée.
- Le sommaire est extrait par introspection et permet d'aller à une page
  (`src/typst/render.rs:109-143`, `src/ui/preview.rs`).
- En revanche, `page_view` ne contient qu'un `image` et un fond de papier :
  **aucune couche de texte, aucun hit-test, aucune sélection**.
- `pdf-render` est un rasteriseur : son API publique rend des pages ou des
  régions en pixels ; elle n'expose ni texte, ni recherche, ni liens
  (`vendor/pdf-render-1.3.0/README.md` et `src/typst/render.rs:486-523`).
- `Ctrl+F` ouvre aujourd'hui la recherche **de l'éditeur**, pas celle du
  document composé (`src/app/mod.rs:2161`, `docs/guide/recherche.md`).
- Les liens existent dans les cadres Typst (`FrameItem::Link`), mais le
  lecteur ne les utilise pas : ils sont perdus à l'affichage.

Autrement dit, les données nécessaires à la sélection et à la recherche
**sont déjà en mémoire**, mais l'aperçu ne les consomme pas.

## 2. Recherches effectuées

### 2.1 Le modèle de Typst est le bon point d'appui

Typst expose, dans `typst-library`, tout ce qu'il faut pour reconstruire une
couche de texte sans relire le PDF :

- `Frame::items()` donne les éléments d'une page avec leur position ;
- `GroupItem` porte une transformation et un cadre imbriqué ;
- `TextItem` porte le texte agrégé, la police, la taille et une liste de
  `Glyph` ;
- `Glyph` porte l'avance, les décalages, la plage dans le texte de l'item et
  un `Span` de source ;
- `TextItemView::glyph_text()` donne le texte d'un glyphe, y compris les
  ligatures et les caractères multi-octets ;
- `FrameItem::Link` porte une destination (`Url`, `Position` ou `Location`) et
  une taille ;
- `PagedIntrospector::position()` résout une `Location` en page et point.

La géométrie est en points Typst, donc indépendante du zoom et de la densité
de rasterisation. Une couche construite ainsi reste valable quand on zoome,
qu'on change de panneau ou qu'on ne rasterise qu'une région.

### 2.2 Ce que font les lecteurs de référence

**PDF.js** sépare trois couches : le canevas raster, une couche de texte
transparente posée dessus, et un contrôleur de recherche. La couche de texte
sert à la sélection, au copier et à la recherche ; le contrôleur de recherche
parcourt les pages, compte les résultats et demande au lecteur d'afficher le
résultat courant. Le gestionnaire de rendu, lui, donne la priorité aux pages
visibles, puis aux pages voisines dans le sens du défilement, et annule le
rendu devenu inutile.

Sources : `text_layer_builder.js` et `pdf_find_controller.js` de PDF.js,
`pdf_rendering_queue.js` (voir la section Sources).

**pdfium-render** expose une couche texte riche : texte complet, segments avec
boîtes, caractères avec origine, et recherche par page avec options de casse
et de mot entier. C'est la solution la plus complète si l'on accepte PDFium,
mais elle repose sur une bibliothèque native C++ chargée à l'exécution, avec
les contraintes de packaging que cela implique.

**MuPDF** fait encore plus (texte, liens, annotations, formulaires), mais la
crate est sous **AGPL-3.0** ; ce n'est pas compatible avec la distribution MIT
de l'application sans accord commercial.

**pdf-extract** extrait du texte, éventuellement page par page, mais ne donne
pas de coordonnées de glyphes ni de recherche : il ne peut pas porter une
sélection fidèle.

**typst-render** est le rasteriseur officiel de Typst : il rend une `Page` ou
un `PagedDocument` en pixmap avec `tiny-skia`, sans passer par le PDF. Il ne
fournit pas de couche texte (celle-ci vient des cadres), mais il peut servir à
produire un aperçu rapide ou un placeholder avant le rendu PDF fin.

### 2.3 Tableau de décision

| Solution | Texte et boîtes | Recherche | Liens | Coût / risque |
|---|---|---|---|---|
| **Cadres `PagedDocument`** | Oui, glyphes, positions, `Span` | À construire (petit moteur) | Oui, `FrameItem::Link` | Aucune dépendance nouvelle ; données déjà chargées ; à valider contre le PDF |
| `pdfium-render` | Oui, très complet | Oui, native | Oui | Bibliothèque native C++ à empaqueter ; remplace ou double le moteur de rendu |
| `pdf-extract` | Texte seul | À construire, sans géométrie | Non | Insuffisant pour la sélection |
| `mupdf` | Oui, très complet | Oui | Oui | AGPL-3.0, lourde chaîne C/C++ |
| `pdf-render` actuel | Non | Non | Non | Excellent raster, mais ne sait faire que cela |
| `typst-render` | Non (mais cadres disponibles) | Non | Non | Utile comme raster rapide, pas comme couche texte |

**Recommandation principale : construire la couche texte depuis
`PagedDocument`, pas depuis le PDF.** C'est la seule solution qui donne à la
fois les positions exactes, les liens, les `Span` de source et l'absence de
dépendance native, tout en gardant `pdf-render` comme source visuelle du PDF
livré.

## 3. Architecture recommandée

### 3.1 Une couche texte dérivée du document compilé

Ajouter un module, par exemple `src/typst/texte.rs`, qui transforme un
`PagedDocument` en une structure indépendante du rendu :

```rust
pub struct DocumentTexte {
    pub revision: u64,
    pub pages: Vec<PageTexte>,
}

pub struct PageTexte {
    pub glyphes: Vec<GlyphePlace>,
    pub lignes: Vec<LigneTexte>,
    pub liens: Vec<LienPlace>,
    pub texte: String,              // texte de lecture, normalisé
    pub ancres: Vec<AncreTexte>,    // index texte -> glyphe
}

pub struct GlyphePlace {
    pub texte: String,
    pub rect: Rect,                 // en points de page
    pub ligne: usize,
    pub span: Span,                 // remontée vers la source .tw
}

pub struct LienPlace {
    pub rect: Rect,
    pub destination: Destination,
}
```

Construction :

1. parcourir récursivement `page.frame.items()` en composant les
   transformations des `GroupItem` ;
2. pour chaque `TextItem`, avancer un curseur sur les `glyphs`, calculer la
   boîte d'avance et, si nécessaire, la boîte exacte de la police ;
3. utiliser `glyph.range()` et `TextItemView::glyph_text()` pour ne pas
   casser les ligatures ni les caractères composés ;
4. regrouper les glyphes en lignes par proximité de ligne de base, puis
   ordonner les lignes et les glyphes dans l'ordre de lecture ;
5. conserver les `FrameItem::Link` avec leur rectangle ;
6. construire le texte de recherche en insérant des séparateurs de ligne et
   une table de correspondance vers les glyphes.

La couche doit être **immuable et partagée** (`Arc<DocumentTexte>`), calculée
une fois par compilation, de préférence hors du fil d'affichage. Les pages
peuvent être extraites à la demande pour ne pas payer les 500 pages d'une
thèse avant la première sélection ; la recherche demande toutes les pages,
mais peut progresser page par page.

### 3.2 Le point délicat : PDF affiché et texte compilé

`RenderState` contient à la fois `document` (mis à jour dès la compilation) et
`lecteur` (le PDF, produit après l'export). Pendant l'export, l'écran montre
encore l'ancien PDF. Il faut donc :

- étiqueter `DocumentTexte` par la **révision d'affichage** du PDF, pas
  seulement par la révision du document ;
- garder l'ancienne couche texte tant que l'ancien PDF est visible ;
- remplacer les deux ensemble quand le nouveau PDF arrive ;
- refuser toute sélection ou recherche dont la révision ne correspond pas à
  l'image affichée.

C'est le même principe que le double tampon des images, appliqué au texte.
Sans cette règle, on pourrait sélectionner une phrase du nouveau document sur
une page de l'ancien.

### 3.3 Sélection

La sélection est un **overlay** au-dessus de l'image, pas une modification de
l'image :

- un `canvas::Program` (ou un widget maison) posé sur la page ;
- au `ButtonPressed`, trouver le glyphe le plus proche et poser l'ancre ;
- au `CursorMoved` tenu, étendre jusqu'au glyphe sous le curseur ;
- au `ButtonReleased`, figer la sélection ;
- double-clic : mot ; triple-clic : ligne ou paragraphe ; `Maj+clic` :
  étendre depuis l'ancre ;
- dessiner les rectangles de ligne sélectionnés avec une couleur translucide
  du thème, jamais un rectangle par glyphe (coût visuel et graphique) ;
- `Ctrl+C` et menu contextuel : copier le texte dans l'ordre de lecture ;
- sélection multi-pages : ordre global `(page, ligne, glyphe)` ; le drag peut
  traverser l'espace entre deux pages ;
- pendant un recompilation, conserver la sélection seulement si les ancres
  existent encore ; sinon la retirer explicitement.

Pour le copier, commencer par du **texte brut** : lignes jointes par `\n`,
espaces reconstruits entre les mots, césures de fin de ligne traitées selon
la politique choisie. Le copier riche (HTML, Typst) est une seconde étape.

Le survol d'une page ne doit pas capturer la molette : le `MoletteDirigee`
existant continue de diriger `Ctrl+molette` vers le zoom et le reste vers le
défilement. Il faudra le vérifier explicitement avec l'overlay.

### 3.4 Recherche

Créer une recherche **du document rendu**, distincte de la recherche de
l'éditeur :

- `Ctrl+F` dans l'aperçu ouvre la barre de recherche PDF ; dans l'éditeur, il
  garde son sens actuel. Le critère doit être le panneau actif (le mode de la
  vue active, `ModeVue::Rendu`), pas une supposition globale ;
- champ de recherche, compteur `3/17`, précédent/suivant, `Entrée`, `F3`,
  `Maj+F3`, `Échap` ;
- recherche littérale par défaut, insensible à la casse et aux accents
  (normalisation NFKD ou table de pliage français), avec une option « mot
  entier » et, plus tard, une option expression régulière si elle est
  réellement utile ;
- surligner tous les résultats, distinguer le résultat courant, faire défiler
  l'aperçu jusqu'à lui et le rendre visible même si sa page n'est pas encore
  rasterisée ;
- compteur global et compteur par page ; si l'index est encore en cours de
  construction, afficher un état « recherche… » plutôt qu'un faux zéro ;
- relancer la recherche après une compilation, en conservant le motif, ou
  l'invalider clairement si le document a changé.

Le texte cherché est celui du **document composé** : titres générés,
sommaire, bibliographie et notes comprises. C'est ce que l'auteur voit dans
le PDF, et c'est donc ce qu'il attend d'un `Ctrl+F` dans l'aperçu.

Pour la performance : index par page, texte normalisé + table de
correspondance, recherche dans un thread de travail, résultats renvoyés par
messages. Sur une thèse de 500 pages, l'index ne devrait être construit
qu'une fois par compilation, pas à chaque frappe.

### 3.5 Liens et navigation

Les liens sont déjà dans les cadres. Il faut :

- poser un rectangle cliquable sur chaque `FrameItem::Link` ;
- `Destination::Url` : ouvrir le navigateur par le mécanisme déjà utilisé
  pour les liens du projet (`xdg-open` sur Linux, avec la même politique de
  sécurité) ;
- `Destination::Location` : résoudre la `Location` avec l'introspecteur,
  puis défiler vers la page et le point ;
- `Destination::Position` : page et point directs ;
- changer le curseur en main au survol, afficher l'URL en infobulle ;
- ne pas confondre un clic de lien avec un début de sélection : un clic sans
  déplacement sur un lien ouvre, un drag sélectionne.

Ajouter ensuite les gestes de lecteur qui manquent :

- `Ctrl+G` ou un champ « page » pour aller à une page ;
- `Début` / `Fin`, `Page précédente` / `Page suivante` ;
- historique précédent/suivant après un saut depuis le sommaire, un lien ou
  une recherche ;
- éventuellement, un clic droit « Voir dans le texte » grâce aux `Span` des
  glyphes — c'est la fonctionnalité qui transforme l'aperçu en outil de
  travail pour l'auteur, mais elle demande la même table de correspondance
  que le sommaire vers les fichiers `.tw`.

### 3.6 Points d'intégration dans l'existant

Pour que le rapport soit directement utilisable comme plan de travail :

- `RenderState` : ajouter `texte: Option<Arc<DocumentTexte>>`,
  `selection: Option<SelectionApercu>` et `recherche: Option<RechercheApercu>`.
  La couche texte et la sélection portent la révision du PDF affiché.
- `Message` : `TexteApercuPret(revision, Arc<DocumentTexte>)`,
  `SelectionApercuCommencer`, `SelectionApercuEtendre`,
  `SelectionApercuTerminer`, `SelectionApercuCopier`,
  `RechercheApercuOuvrir`, `RechercheApercuMotif`, `RechercheApercuSuivant`,
  `RechercheApercuPrecedent`, `LienApercuActiver`.
- `src/ui/preview.rs` : `page_view` passe d'un `container(image)` à un
  `stack![image, canvas]` (ou un overlay équivalent) ; le canevas ne dessine
  que les surlignages et ne recopie jamais le texte.
- `src/app/rendu.rs` : construire `DocumentTexte` après une compilation
  réussie ; ne l'installer qu'avec le PDF correspondant ; jeter l'ancien
  couple (image, texte) en bloc.
- `src/app/saisie.rs` : router `Ctrl+F` selon la vue active
  (`ModeVue::Rendu`) ; garder `Recherche` pour l'éditeur et créer une
  recherche distincte pour l'aperçu.
- `src/typst/render.rs` : exposer un extracteur pur, testable sans interface,
  qui prend `&PagedDocument` et rend `DocumentTexte`. C'est lui qui porte les
  tests de glyphes, de lignes et de liens.

### 3.7 Fluidité

Le lecteur actuel a déjà les bons fondamentaux : rendu par région, densité
d'écran, cache borné, `Handle` stable, double tampon, zoom différé. Il ne faut
pas le réécrire ; il faut supprimer les travaux inutiles.

**Priorité 1 — ordonnancer et annuler.**

Aujourd'hui `rasterisation::visible()` construit une tâche pour chaque page
visible périmée et les lance toutes en `Task::batch`. Une page ancienne peut
encore être en cours quand la révision change ; son résultat est jeté, mais
son coût CPU est payé. Introduire une file de travaux :

- une priorité : pages visibles d'abord, puis une page de prélecture dans le
  sens du défilement ;
- au plus un ou deux travaux simultanés (le rasteriseur a déjà son propre
  parallélisme) ;
- une génération de révision : tout travail d'une génération dépassée est
  abandonné avant de commencer, et son résultat est ignoré ;
- coalescer les changements de viewport : un défilement de 60 événements par
  seconde ne doit pas créer 60 files.

C'est exactement la politique du `PDFRenderingQueue` de PDF.js : pages
visibles, prélecture directionnelle, annulation, priorité.

**Priorité 2 — qualité progressive (décision finale).**

La qualité progressive a été **abandonnée** après mesure : elle coûtait
systématiquement deux rasterisations par page (vitesse puis qualité), soit
environ le double du travail pendant un défilement, pour un écart
sous-perceptuel. L'aperçu utilise donc **une seule passe
`RasterQuality::Speed`** ; la qualité `Quality` reste celle de l'export et des
vignettes qui exigent le pipeline f32.

**Priorité 3 — tuiles pour les forts zooms.**

Aujourd'hui le cache contient une **tranche par page**. À 500 %, une tranche
visible peut peser plusieurs dizaines de mégaoctets ; un petit défilement
hors de la tranche relance toute la page. Un cache de tuiles (par exemple
512 × 512 ou 1024 × 1024 pixels) avec LRU :

- ne rend que les tuiles manquantes ;
- partage les tuiles entre défilements ;
- borne la mémoire indépendamment de la taille de la page ;
- se marie bien avec le rendu par région déjà présent.

C'est un changement plus lourd ; il se justifie si les mesures montrent que
les re-rendus de tranche sont fréquents sur les documents longs ou aux forts
zooms. Pour un premier lot, la file prioritaire, la plage exacte et la passe
unique `Speed` suffisent souvent.

**Priorité 4 — premier affichage.**

La première page reste un cadre papier tant que le PDF n'est pas exporté et
rasterisé. Deux options :

- rendre un **placeholder basse résolution** de la page entière (PDF ou
  `typst-render`) avant la région fine ;
- conserver et réafficher l'ancienne texture à l'échelle pendant le zoom,
  ce qui est déjà le cas pour les pages déjà vues.

Le placeholder améliore le temps perçu, pas la qualité finale : il ne doit
jamais remplacer une image fine déjà présente.

**Mesures à ajouter avant de conclure :** temps du premier pixel, temps de
rasterisation par région, nombre de `requested` par seconde de défilement,
travaux abandonnés, octets de textures, mémoire du cache. Le compteur
`RenderState.requested` existe déjà ; il faut le compléter, pas le remplacer.

## 4. Ce qu'il ne faut pas faire maintenant

- **Ne pas ajouter PDFium ou MuPDF pour lire notre propre PDF.** Ils
  apporteraient la sélection et la recherche, mais au prix d'une bibliothèque
  native, d'un packaging multiplateforme et, pour MuPDF, d'une licence
  incompatible. Nous avons déjà le document Typst en mémoire.
- **Ne pas extraire le texte avec `pdf-extract` pour la sélection.** Il ne
  donne pas les boîtes des glyphes ; il ne pourrait servir qu'à un index de
  recherche approximatif, et ferait un second parseur pour rien.
- **Ne pas passer l'aperçu à `typst-render` comme source principale.** Le
  lecteur doit montrer le PDF livré ; changer de moteur changerait la
  fidélité. `typst-render` peut être un placeholder ou un outil de diagnostic.
- **Ne pas ajouter l'édition d'annotations, les formulaires ou l'OCR.** Ce
  n'est pas le métier de l'application : elle écrit des documents Typst, elle
  ne remplace pas Acrobat.

## 5. Découpage proposé

| Lot | Contenu | Taille | Dépend de |
|---|---|---|---|
| A | Couche `DocumentTexte`, révision d'affichage, extraction des glyphes et liens | M | — |
| B | Sélection, surlignage, copier, double/triple-clic, multi-pages | M | A |
| C | Recherche PDF, barre, compteur, surlignage, défilement vers le résultat | M | A |
| D | Liens cliquables et navigation interne/externe | S | A |
| E | File de rendu prioritaire, annulation des générations dépassées, plage exacte et passe unique Speed | M | — (parallèle) |
| F | Tuiles et placeholder basse résolution, si les mesures le justifient | L | E |
| G | « Voir dans le texte » depuis un glyphe, historique, vignettes | M | A, D |

Ordre conseillé : **A, puis B et C en parallèle, puis D, puis E**. E peut
commencer indépendamment et améliorer immédiatement la sensation de fluidité.
F ne se décide qu'après les mesures de E.

## 6. Vérifications et critères d'acceptation

Tests automatiques (HOME jetable, projet synthétique, jamais le projet réel
de l'auteur) :

- extraction : une ligne avec ligature, une ligne avec accents, une note de
  bas de page, un titre, un lien interne et un lien externe ;
- sélection : un mot, une phrase à l'envers, une sélection traversant deux
  pages, copier-coller exact ;
- recherche : plusieurs occurrences, casse et accents, résultat sur une page
  non encore rasterisée, aucun résultat, recompilation pendant la recherche ;
- liens : lien interne vers la bonne page, lien externe non ouvert par un
  simple drag ;
- rendu : `requested` borné pendant un défilement simulé, aucune page vide,
  cache jamais négatif.

Validation visuelle réservée à l'auteur, selon la règle du projet : sélection
au glisser, double-clic, copier, barre de recherche, surlignage, lien interne,
zoom et défilement sur un document long.

Critères de succès :

1. sélectionner une phrase dans l'aperçu et la coller donne exactement le
   texte attendu ;
2. `Ctrl+F` dans l'aperçu trouve le texte du document composé, pas celui de
   la source, et compte les résultats ;
3. un lien interne du PDF amène à la bonne page ;
4. un défilement rapide ne déclenche pas de vague de rasterisations
   inutiles ;
5. aucune régression de fidélité : l'image affichée reste le PDF exporté.

## 7. Risques et limites

- **Deux sources.** Le texte vient des cadres Typst, l'image du PDF. Ils
  doivent être liés à la même révision ; sinon la sélection dérive. C'est le
  point à tester en premier.
- **Ordre de lecture.** Les cadres sont en ordre de peinture ; les colonnes,
  les notes et les en-têtes demandent un tri par lignes et un traitement
  particulier. Commencer par une seule colonne, puis ajouter les cas.
- **Ligatures, compositions, mathématiques.** Utiliser les plages de glyphes,
  pas les indices de caractères ; les formules et les images peuvent ne pas
  être sélectionnables, et il faut le dire plutôt que produire du texte faux.
- **Performance de l'index.** Ne pas construire un index complet à chaque
  frappe ; le faire par compilation et par page, avec annulation.
- **Mémoire.** Les tranches à fort zoom sont grosses ; les tuiles ou un
  plafond de pixels sont la soupape.
- **Liens externes.** Ouvrir une URL est une action extérieure : garder le
  mécanisme existant, ne pas accepter de schéma douteux, et ne pas ouvrir un
  lien pendant un drag de sélection.

## 8. Sources

### Dans le dépôt

- `src/app/state.rs` — `RenderState`, `document`, `lecteur`, cache et
  révisions.
- `src/ui/preview.rs` — géométrie, pages, `page_view`, régions.
- `src/app/rasterisation.rs` — sélection des pages visibles et lancement des
  rasterisations.
- `src/typst/render.rs` — compilation, PDF, `render_region`, sommaire.
- `src/app/rendu.rs` — cycle compilation → PDF → cache de pages.
- `vendor/pdf-render-1.3.0/README.md` et `PATCHES.md` — rasterisation pure
  Rust, rendu par région.
- `typst-library` 0.15.1, `layout/frame.rs`, `text/item.rs`,
  `model/link.rs` — `Frame`, `TextItem`, `Glyph`, `GroupItem`,
  `FrameItem::Link`, `Destination`.

### Extérieur

- PDF.js, `text_layer_builder.js` :
  <https://github.com/mozilla/pdf.js/blob/master/web/text_layer_builder.js>
- PDF.js, `pdf_find_controller.js` :
  <https://github.com/mozilla/pdf.js/blob/master/web/pdf_find_controller.js>
- PDF.js, `pdf_rendering_queue.js` (priorité, prélecture, annulation) :
  <https://github.com/mozilla/pdf.js/blob/master/web/pdf_rendering_queue.js>
- `pdfium-render` (texte, segments, recherche, bibliothèque native) :
  <https://docs.rs/pdfium-render/latest/pdfium_render/>
- `pdf-extract` (texte seul, sans géométrie) :
  <https://docs.rs/pdf-extract/latest/pdf_extract/>
- `mupdf` (AGPL-3.0) :
  <https://docs.rs/crate/mupdf/latest>
- `typst-render` (raster officiel d'une `Page`/`PagedDocument`) :
  <https://docs.rs/typst-render/latest/typst_render/>
- Typst, `PagedDocument` et introspection :
  <https://docs.rs/typst-layout/0.15.1/typst_layout/struct.PagedDocument.html>
