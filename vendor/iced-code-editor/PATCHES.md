# iced-code-editor — copie vendorisée

Ce dossier est une copie de `iced-code-editor` 0.5.2, dans laquelle ce projet
apporte **dix modifications**. Le reste du fichier est la version amont.

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

## Les treize modifications

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

### 4. Touches réservées à l'hôte (`reserved_keys`)

**Pourquoi.** Une liste de complétion a besoin des flèches, d'Entrée, de
Tabulation et d'Échap. L'éditeur les consomme, donc l'hôte ne les voit jamais —
et les flèches déplaceraient le curseur *en plus* de la sélection.

**Quoi.** Un champ `reserved_keys`, une méthode `set_reserved_keys`, et huit
lignes dans `handle_keyboard_event` : une touche réservée est *déclinée* plutôt
que traitée. Un événement qu'un widget décline continue son chemin, et l'hôte le
reçoit.

**Ce qui continue de marcher.** Tout le reste : on continue de taper, la liste
peut donc se filtrer à la frappe, et un retour arrière reste un retour arrière.

**Ce qui disparaît si l'on revient à l'amont.** La liste de complétion ne peut
plus être pilotée au clavier ; il ne reste que la souris.

### 3. Palette de coloration syntaxique fournie par l'application

`src/canvas_editor/render/highlighting.rs` et `src/canvas_editor/mod.rs`.

Le crate choisissait sa palette de coloration en dur, par **nom**, dans le
`ThemeSet` de syntect : `base16-ocean.dark` ou `base16-ocean.light` selon la
clarté du fond, sans aucun moyen d'en fournir une autre. Un thème de
l'application ne pouvait donc décrire la coloration, et la rattacher au reste de
la fenêtre était impossible.

Sont ajoutés :

- `SyntaxRule` — un sélecteur de portée TextMate et une couleur ;
- `SyntaxPalette` — la couleur du texte ordinaire, le fond, et les règles
  **dans l'ordre d'application** ;
- `CodeEditor::set_syntax_palette(Option<SyntaxPalette>)` — `None` revient au
  comportement d'origine.

La palette fournie est construite en `syntect::highlighting::Theme` et primée
sur la recherche par nom. Elle est conservée en `&'static` (fuite assumée, une
fois par changement de thème) parce que tout le chemin de résolution manipule
déjà des références statiques ; la faire tenir autrement aurait demandé de
reprendre la structure du cache `ResolvedSyntax`. Poser une nouvelle palette
vide ce cache, sans quoi l'ancienne resterait en mémoire.

L'ordre des règles est délibérément sous la responsabilité de l'application :
syntect compare les portées **par préfixe** et retient la dernière règle qui
correspond, si bien qu'une règle générale doit précéder une règle fine —
`constant` avant `constant.numeric`, `keyword` avant `keyword.operator`.

### 5. Changer de palette invalide la coloration déjà calculée

`src/canvas_editor/render/highlighting.rs`, fonction `set_syntax_palette`.

Le cache de coloration garde des couleurs **résolues**, et non les portées qui
les ont produites : changer la palette sans vider ce cache laisse donc à
l'écran les couleurs de l'ancienne. `set_theme` fait déjà l'invalidation ;
`set_syntax_palette`, écrit juste après lui, l'avait oubliée — et la
coloration ne dépendait pas du thème choisi.

Le remède est la même invalidation, au même endroit que dans `set_theme`.

### 6. Texte sélectionné exposé à la barre de mise en forme

`src/canvas_editor/editing/selection.rs`, méthode `CodeEditor::selected_text`.

L'éditeur sait extraire le texte sélectionné pour ses propres commandes, mais ne
l'exposait qu'à l'intérieur du crate. La barre de mise en forme du projet doit
pouvoir entourer la sélection avec le balisage de l'AST ; cette méthode publique
renvoie donc le texte du curseur primaire, ou `None` sans sélection. Elle
réutilise l'extraction Unicode existante, sans dupliquer les calculs d'octets.

### 7. Auto-pairage des apostrophes configurable indépendamment

`src/canvas_editor/mod.rs`, `src/canvas_editor/bool_options.rs`,
`src/canvas_editor/config.rs` et `src/canvas_editor/input/update/text_input.rs`.

Le réglage historique traitait les apostrophes comme les parenthèses. En français,
`l'` devenait `l''`, ce qui obligeait l'utilisateur à franchir un caractère
fantôme ; désactiver toutes les paires rendait en revanche les liens Markdown,
notes et couleurs plus pénibles à écrire. Le nouveau booléen indépendant permet
aux documents `.tw` d'auto-fermer les crochets, parenthèses et accolades tout en
laissant `'` littéral. Les autres formats gardent leur comportement existant.

### 8. Le texte de secours du surligneur reste opaque

`src/canvas_editor/render/highlighting.rs`.

Pour borner le temps de rendu des gros fichiers, syntect ne traite qu'un nombre
limité de lignes par image. Une ligne profonde pas encore colorée retombait sur
`style.text_color` ; avec un thème où cette couleur est transparente, le texte
était sélectionnable mais invisible. Le secours prend maintenant la couleur de
texte de la palette syntaxique (et force son opacité), comme les lignes déjà
colorées. Un test épuisant le budget vérifie ce cas.

### 9. Largeur réelle du canvas exposée à l'interface hôte

`src/canvas_editor/metrics.rs`, méthode `CodeEditor::canvas_width`.

`viewport_width` ne reçoit parfois sa mesure réelle qu'au premier événement de
défilement. L'interface hôte en a besoin plus tôt pour borner ses bulles au
panneau et pouvait donc les placer hors écran dans une fenêtre étroite. La
nouvelle lecture renvoie la dernière largeur observée par le dessin du canvas.

### 10. Les notes fondues en un glyphe

`src/canvas_editor/marques.rs` (nouveau), `render/text.rs`,
`render/wrapping.rs`, `editing/cursor.rs`, `render/overlays.rs`, `caches.rs`,
`theme.rs`.

Une citation groupée est **une** note, une note libre `^[…]` en est une
autre : l'application demande qu'elles s'écrivent d'un petit numéro en
exposant, à la place du texte qu'elles portent. Le texte reste le vrai —
c'est la mise en page qui le tait.

Une *marque* est une plage de colonnes de sa ligne logique. Sous elle, les
caractères ne pèsent rien, et le premier porte la largeur du glyphe : cette
seule règle de largeur (`largeur_caractere`) réfracte l'habillage, la
géométrie des surlignages, la position du curseur et la conversion des clics,
qui passent tous par là. Le dessin marche en *bouts* — ce qui se lit, puis le
glyphe — pour que le texte fondu sorte du tracé sans laisser de trou.

La prise `set_marques(Vec<Marque>)` porte les plages et fait monter une
révision qui invalide le cache d'habillage ; la lecture `marques()` rend
les plages à l'hôte, pour qu'il ne les repose que si elles ont changé. La couleur des glyphes est
`Style::mark_color`. Les tests du module prouvent la règle de largeur.

## La feature `two-face` est nécessaire pour tester

Les tests de ce crate **doivent** être lancés avec la feature, sinon il n'y a
pas de grammaire Typst :

```sh
cargo test --lib --features two-face
```

Sans elle, `find_syntax_by_extension("typ")` ne trouve rien et tout retombe
silencieusement sur du texte brut : les tests de coloration passent à vide, et
l'on croit avoir vérifié quelque chose. C'est exactement le piège qui a fait
échouer la première version du test de palette.

Pour la même raison, l'identifiant de syntaxe est **`typ`**, pas `typst` :
`find_syntax_by_extension("typst")` ne trouve aucune définition et replie sur du
texte brut sans le dire.

## Reprendre une nouvelle version amont

Les neuf modifications sont localisées dans les fichiers concernés, à des
endroits identifiables par les commentaires qui les entourent. Pour passer à une
version `N+1` :

1. remplacer le contenu de ce dossier par la nouvelle version amont ;
2. réappliquer les neuf modifications ci-dessus ;
3. vérifier que les tests ajoutés passent depuis ce dossier :
   `cargo test --lib --features two-face`.

Les tests de ce crate **ne sont pas exécutés** par le `cargo test` de la racine :
la copie n'est pas un membre de l'espace de travail, car l'ajouter en membre
ferait entrer ses dépendances dans la résolution du projet. Il faut donc les
lancer depuis ce dossier.

## Licence

MIT, comme le crate amont. Le fichier `Cargo.toml.orig` de la version amont est
conservé pour référence.
### 11. La note est un caractère entier, et le clic est sa seule porte
Une note fondu en un glyphe ne doit pas rester un **fantôme** : on la
touchait par derrière, au clavier, et son texte invisible se copiait, se
remplaçait, s'éditait au fil de la frappe. Elle est devenue un caractère
entier. Le curseur **la saute d'un bond** (flèches) et **atterrit sur ses
bords** (haut/bas, clic) ; la sélection la couvre d'un bloc ; Backspace et
Suppr **l'emportent entière** au lieu de la grignoter. Et un clic dessus est
**la seule porte d'édition** d'une citation — la frappe n'ouvre plus jamais le
composeur — pendant que le pointeur, sur le glyphe, devient **cliquable**
(`mouse::Interaction::Pointer`). Au moteur : `Marque::commençant_a` /
`finissant_a`, `CodeEditor::marque_au_point(Point)`, `dernier_clic_marque` —
un clic sur un glyphe est mémorisé, l'hôte décide seul d'ouvrir quoi. Le
pied de page, lui, se déplie sous le curseur pour s'y écrire, et replie dès
qu'on le quitte.

### 13. La note entière est un groupe, et le collage est une étape
Une note sur plusieurs lignes avait une marque par rangon, mais les gests ne
voyaient que le rangon sous le curseur : Backspace en emportait la moitié, et
l'autre moitié restait orpheline. `Marque` porte maintenant un `groupe` — la
note entière — et `bords_du_groupe` donne ses deux bords : la suppression,
les flèches et les sauts traitent le groupe comme **un** caractère. Autre
piège voisin : coller sur une sélection faisait deux commandes (suppression,
puis insertion), et Ctrl+Z ne reprenait que la seconde. `paste_text` groupe
les deux en une seule étape d'annulation.

`marques.rs` (`groupe`, `bords_du_groupe`), `editing/cursor.rs`,
`input/update/deletion.rs`, `editing/clipboard.rs`.

### 12. La sélection remplace, les retours arrière ne comptent plus
Un remplacement bâti en N retours arrière est un piège : N se comptait en
**octets** là où chaque frappe efface un **caractère**, et la suppression en
caractère entier (modification 11) fait qu'une note part en une seule
frappe — tout le reste des retours rongeait le texte d'avant. Le crate
gagne `CodeEditor::selectionner(debut, fin)` : la plage devient une
sélection, que le collage remplace d'un geste. Aucun comptage, aucune
unité à égaliser.

`editing/cursor.rs` (`selectionner`).


## La recherche du crate est désactivée

`search_replace_enabled` passe à `false` à la construction : la recherche
propre au crate — sans expressions régulières ni rappel de groupes —
laissait `Ctrl+F` sans emploi pour celle de l'application, qui en a. Le
raccourci est donc libre, et la recherche de l'application le reprend.
