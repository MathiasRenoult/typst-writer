# Journal des versions

## 0.3.0-alpha — suite : le rapport 4 soldé, la qualité au sommet

Le rapport de bugs 4 (63 demandes d'interface et de qualité) est
intégralement traité. Nettoyage complet du code et du vendor.

### Le langage

- **saut de page** : trois lignes vides consécutives → `#pagebreak()` ;
- **souligné et barré** : `__texte__` et `~~texte~~` ;
- **commentaires** : `// texte` — conservé dans le `.tw`, ignoré au rendu ;
- **autoformatage des tableaux** : les lignes du corps sont normalisées
  au nombre de colonnes de l'en-tête.

### La recherche

- **mode littéral / regex** : bouton `Aa` / `.*` — le texte est pris au
  pied de la lettre ou interprété en expression régulière ;
- **surlignage des occurrences** dans l'éditeur à chaque frappe.

### Les normes

- **Quick Parts** : `{document}`, `{date}`, `{auteur}`, `{projet}` dans
  l'en-tête et le pied de page — dynamiques, jamais figés ;
- **casse par niveau de titre** : chaque niveau (1-6) a sa propre casse ;
- **aperçu de l'imbrication des guillemets** en temps réel.

### L'interface

- **sommaire en arborescence** : lignes de connexion `│ ├── └──` ;
- **barre supérieure** : sélecteur de police, pastille de couleur
  courante, séparateurs par famille de commandes, icônes alignées ;
- **éditeur de thème** : sections (Fond / Texte / États), color picker
  natif (zenity), infobulle sur la pastille ;
- **panneau latéral** : bouton de réduction dans le panneau, année de
  publication dans la bibliothèque, filtre par année ;
- **espace de travail** : `Ctrl+Maj+Tab` bascule de panneau, dépôt
  d'onglet hors panneau = nouveau panneau ;
- **mode focus** : le rendu est centré.

### La qualité

- **clippy 0** dans le projet et le vendor ;
- **nettoyage du vendor** : imports explicites, `enumerate()` au lieu de
  compteurs manuels, `#[allow]` documentés ;
- **comptage bibliographique fiabilisé** : une entrée tronquée n'emporte
  plus les suivantes ;
- `PATCHES.md` : 17 points documentés.

## 0.3.0-alpha — l'espace de travail, la bibliothèque, et les trois rapports soldés

La version où l'éditeur est devenu un **espace de travail** — panneaux
divisibles, onglets par panneau, disposition restaurée —, où la
bibliothèque de projets prend toute la fenêtre, où les graphiques
entrent dans la langue, et où les **trois rapports de bugs sont
soldés** (18 + 14 + 17 points).

### L'espace de travail

- **Panneaux divisibles** : l'éditeur se scinde en autant de panneaux
  qu'on veut (bord de 8 px), chacun avec ses onglets ; glisser un onglet
  le déplace, le détache ou le divise ;
- **onglets par panneau**, nommés par projet (`document.tw` → le nom du
  projet) ; un point de mode coloré dit l'état (bleu édition, vert
  rendu, orange Typst) ; le × apparaît au survol ;
- **même document, plusieurs vues** : texte partagé, vues indépendantes
  (édition / rendu / Typst) — chacune scrolle et zoome à son gré ;
- **disposition restaurée** : panneaux, onglets, modes et documents se
  retrouvent au lancement ; les vues orphelines ne sont pas sérialisées ;
- **page d'accueil** dans les panneaux vides : Créer, Ouvrir, Bibliothèque ;
- **shift+clic** = nouvel onglet, **clic simple** = remplace l'onglet actif
  (sauvegarde avant fermeture) ;
- **focus mode** maximise la vue active (édition, rendu ou Typst).

### La bibliothèque

- **galerie plein écran** : les projets se voient par couvertures
  cliquables, rangés sur des étagères ;
- **étagères** : classement des projets par drag-and-drop,
  `etageres.json` ; bouton « + Nouveau projet ici » en fin de rangée ;
- les couvertures sont des poignées d'image stables (empreinte de
  pixels), jamais recréées.

### La langue

- **graphiques** : `{graphique type=barre titre="Titre"}` suivi de lignes
  `label | valeur` — trois types (barre, ligne, secteur), rendus par le
  paquet Typst `cetz` ; l'import n'est ajouté que si le document
  contient un graphique ;
- **légende d'image par `|`** : `![alt](chemin | légende)` — la légende
  emporte sur l'alt, numérotée par les normes ;
- **notes de bas de page** : clic sur `^[]` → overlay en bas de l'écran,
  Entrée enregistre, Échap ferme — sans déplier la note ni déranger le
  fil du document.

### L'interface

- **éditeur de thème complet** : ouverture sur le thème actif, application
  immédiate, bouton « Réinitialiser » (recharge depuis le disque) ;
- **Ctrl+1-6** insère un titre au curseur (`#`×n + espace), jamais de
  reformatage de bloc ;
- **molette et flèches** sur les champs numériques des normes (la valeur
  garde son unité : `11pt` → `12pt`, `2cm` → `3cm`) ;
- **en-tête et pied de page** dans les normes : contenu libre,
  numérotation multiple (a, A, 1, p. 1, page 1 sur 12, 1/12…) ;
- **icône titre hover** : menu H1-H6 au survol ;
- **liseré actif** limité à la barre d'onglets ;
- **boutons inutiles supprimés** (renommer symbole, position aperçu) ;
- **clavier de la fiche de référence** : Tab/Entrée/Échap ne sont plus
  avalés par le composeur fermé ;
- **contraste renforcé** : `text_muted` calculé par mélange de
  luminance, tenue dans tous les thèmes ;
- **mode Typst** : texte généré complet (pipeline `assembler`), plus de
  `#include` seul.

### La qualité

- **`deny.toml` configuré** : 13 licences SPDX permissives autorisées,
  11 advisories documentés avec raison, sources strictes (crates.io
  uniquement) — `cargo deny check` passe (exit 0) ;
- **zéro `unsafe`** dans `src/`, 243 `unwrap()` dont 95 % en tests ;
- **fuite mémoire corrigée** : l'aperçu de police fuyait une `String` à
  chaque frappe (`Box::leak` → `intern_family` avec cache) ;
- **12 `Eq`** ajoutés sur les types qui dérivaient `PartialEq` sans
  raison ; 5 `const fn` ; 5 clones redondants supprimés ;
- **zéro allocation** dans la sélection des polices (`&str` au lieu de
  `String`).

**Comptes** : 395 tests projet, 654 tests du crate vendoré, clippy 0,
`cargo deny` ok.

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
