# Journal des versions

## 0.2.0 — la citation, le langage, et l'interface

La version où le document est devenu une **couche simple** (`document.tw`) entre
les doigts et Typst, où les citations se sont faites **glyphes**, et où
l'interface s'est tue pour laisser parler le texte.

### Le langage intermédiaire

- `document.tw` : titres, citations `@cle[p. 45]`, notes libres `^[…]`, blocs
  bruts pour le reste ; sérialisation canonique, migration des projets `.typ`
  hérités sans rien détruire ;
- la syntaxe, l'AST et les deux traductions sont documentées dans
  `docs/design/langage-intermediaire.md` ; le même document génère toujours le
  même corps Typst.

### Les citations

- les citations deviennent des **glyphes numérotés dynamiques** (¹ ² ³) — appels
  de notes et notes libres mélangés dans un seul compteur, dans l'ordre du
  document ;
- une note est un **caractère entier** : curseur, sélection, suppression ;
- **cliquer sur un glyphe** ouvre le composeur et déplie la note au pied de la
  page ; la frappe n'ouvre jamais l'édition, et `Échap` abandonne sans trace ;
- le composeur se conduit aux flèches (la note, puis les clés) et réécrit chaque
  référence à son rang ;
- correctifs mesurés : le clic tombe où l'on clique malgré les glyphes, la
  frappe après un glyphe reste du texte (elle grossissait la clé), la flèche
  gauche sort d'avant une note, la suppression emporte la note entière et une
  seule annulation reprend tout le geste, le remplacement d'une citation ne
  mange plus le texte d'avant, le glyphe est posé aussitôt l'insertion (plus de
  clé à nu dans le `.tw`).

### Les références

- le panneau latéral liste **auteur, prénom, titre** (abrégés s'il faut), jamais
  les clés ; le crayon ouvre la fenêtre d'édition ;
- la fenêtre réécrit les champs de l'entrée dans `references.bib` — et rien
  d'autre ;
- Zotero : une collection pointée se **lit** ; seules les bibliothèques locales
  s'écrivent.

### L'interface

- **Onze thèmes livrés** — Everforest, Moka, Gruvbox, Pin rosé, Solarized,
  Kanagawa (clairs et sombres) — semés une seule fois, jamais réécrits après,
  avec les thèmes clair et sombre d'origine ; un thème est un petit fichier
  TOML ;
- la **sélection se teinte par la bordure**, jamais par le bloc de couleur ; les
  coins s'arrondissent (rayon 12) ; un seul langage de dessin (jetons de rayon,
  d'espacement et de corps de texte) partout ;
- la fenêtre des normes de mise en page : **infobulles** sur chaque paramètre,
  **listes déroulantes** partout où le choix est fini (langue, numérotation des
  chapitres…) ; toute mesure Typst reste acceptée ;
- le contraste du texte posé sur un fond coloré se **mesure** (luminance), donc
  il tient dans tous les thèmes.

### Robustesse

- gardes fondues depuis le dépôt distant : une clé sans virgule ne garde pas son
  accolade, une étiquette vide ne matche rien, un dossier nommé `references.bib`
  n'est pas une bibliographie ;
- nettoyage : `tower-lsp` et trois fichiers vides retirés ; le chantier des
  inutiles est tenu à chaque release.

**Comptes** : 279 tests projet, 653 tests du crate vendoré, clippy 0.

## 0.1.0-alpha.1 — alpha fermée

La première : écrire en Typst avec le rendu du document final page par page,
l'assistance tinymist (complétion, survol, définition, renommage avec aperçu,
diagnostics), les projets à chapitres, l'export PDF, les thèmes clair/sombre et
le mode focus.
