# iced-code-editor — copie vendorisée

Ce dossier est une copie de `iced-code-editor` 0.5.2, dans laquelle ce projet
apporte **deux modifications**. Le reste du fichier est la version amont.

Le raccordement se fait par `[patch.crates-io]` dans le `Cargo.toml` racine :

```toml
[patch.crates-io]
iced-code-editor = { path = "vendor/iced-code-editor" }
```

## Pourquoi une copie plutôt qu'un réglage

Ces deux modifications ne sont pas exposées par une option du crate. Le
retour à la ligne est découpé au caractère sans qu'aucun réglage ne permette
de le changer, et la liste des serveurs de langage reconnus est une constante
privée. Il n'existe donc aucune voie de configuration : modifier le code est
la seule option.

## Les deux modifications

### 1. Retour à la ligne aux frontières de mots

`src/canvas_editor/render/wrapping.rs`, fonction
`WrappingCalculator::calculate_visual_lines_range`.

La version amont découpe au dernier caractère qui tient dans la largeur :
« aa bbbbbbbb cc » sur une ligne de dix caractères devient « aa bbbbbbb »
puis « b cc ». Le mot est tranché en deux alors que l'espace qui le précède
est un point de rupture tout trouvé — ce qui, en prose, se lit très mal.

La version corrigée, quand la ligne débordet, recule jusqu'à la **dernière
espace du segment** et rompt juste après. L'espace reste à la fin de la ligne
terminée, donc :

- aucun caractère n'est perdu ni dupliqué,
- les colonnes de la ligne logique gardent leur sens, et la correspondance
  curseur ↔ position (clic, flèches, sélection) reste valable sans
  modification.

Cas particuliers conservés :

- un **mot plus long que la ligne** n'a pas d'autre issue que d'être coupé :
  la rupture revient alors au caractère, comme avant ;
- une ligne qui **tient entièrement** produit un seul segment, inchangé ;
- une ligne **vide** produit un segment vide, inchangé ;
- l'**écriture CJK**, sans espaces, se découpe toujours au caractère — c'est
  le comportement attendu pour ces écritures.

Tests ajoutés dans le même fichier, module `tests` :
`test_wrap_at_word_boundary`, `test_no_word_is_cut_across_segments`,
`test_word_longer_than_the_line_is_still_cut`.

### 2. Tinymist comme serveur de langage pour Typst

`src/canvas_editor/lsp/process/config.rs`, deux tables constantes.

Un fichier `.typ` n'était associé à aucun serveur : l'association
extension → langue → serveur ne connaissait que Rust, Python, TypeScript, Lua
et Go. Deux entrées suffisent, le chemin générique de résolution de commande
traitant les clés inconnues (variable d'environnement, puis nom du programme
dans le `PATH`) :

- `LSP_LANGUAGE_MAPPINGS` : `typ` → langue `typst` → serveur `tinymist` ;
- `LSP_SERVER_CONFIGS` : commande `tinymist lsp`, variables `TINYMIST` /
  `TINYMIST_PATH` pour désigner un binaire ailleurs que dans le `PATH`.

## Reprendre une nouvelle version amont

Les deux modifications sont localisées dans deux fichiers, à des endroits
identifiables par les commentaires qui les entourent. Pour passer à une
version `N+1` :

1. remplacer le contenu de ce dossier par la nouvelle version amont ;
2. réappliquer les deux modifications ci-dessus ;
3. vérifier que les tests ajoutés au module `tests` de `wrapping.rs` passent
   depuis ce dossier : `cargo test --lib`.

Les tests de ce crate **ne sont pas exécutés** par le `cargo test` de la racine :
la copie n'est pas un membre de l'espace de travail, car l'ajouter en membre
ferait entrer ses dépendances dans la résolution du projet. Il faut donc les
lancer depuis ce dossier.

## Licence

MIT, comme le crate amont. Le fichier `Cargo.toml.orig` de la version amont est
conservé pour référence.
