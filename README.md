# Typst Writer

Un éditeur pour écrire des documents en Typst — thèses, mémoires, livres.
L'aperçu affiché est celui du document final : c'est le vrai compilateur
Typst qui le compose.

**Version 0.2 — alpha.** Utilisable au quotidien — c'est avec elle
qu'elle s'écrit — mais elle change encore.

## Ce qu'elle fait

- **Écrire** dans une couche markdown lisible, `document.tw` — titres,
  emphase, citations, notes, liens, images, décor — qui se compare ligne
  à ligne, et dont le Typst reste le maître du rendu ;
- **Composer** dès que l'écriture s'arrête : la page se montre telle que
  le lecteur la verra, et l'aperçu suit le chapitre sous le curseur ;
- **Citer** : une bibliographie par projet, une fiche complète (éditeur,
  lieu, titre court, type de document), et un projet lié à une collection
  **Zotero** — lue, jamais réécrite ;
- **Mettre en page** par des normes — papier, marges, titres, couleurs,
  typographie par langue, décor — jamais par des mesures portées dans le
  texte ;
- **Chercher** en expressions régulières, avec remplacement à rappel de
  groupes ;
- **Exporter** en PDF, et travailler au clavier d'un bout à l'autre —
  `F1` montre toutes les touches.

## Écrire

Le document se compose en markdown :

    # Le titre

    Un paragraphe, avec du **gras** et une citation — @dupont[p. 45].

    > une citation de bloc

    {encadré}
    ![La planche](figs/planche.png)

Les mesures de mise en page vivent dans les normes du document ; la
source dit l'intention. La syntaxe entière est dans
[la documentation](docs/guide/ecrire.md).

## Installation

Les versions se téléchargent dans les **publications** : un binaire Linux
x86_64, à décompresser et à lancer tel quel. Il faut un Linux avec
Wayland ou X11, et un pilote graphique offrant Vulkan.

Ou, avec une chaîne Rust à jour :

```sh
cargo build --release
./target/release/typst-writer
```

L'assistance à la frappe s'appuie sur
[tinymist](https://github.com/Myriad-Dreamin/tinymist), qui est
**facultatif** : sans lui, tout le reste fonctionne.

## Documentation

Le manuel d'usage est dans [`docs/guide/`](docs/guide/index.md) — prise
en main, écriture, bibliographie, normes, recherche, raccourcis, fichiers.

## Limites connues

- les diagnostics du compilateur ne sont pas soulignés dans le texte :
  ils sont comptés dans la barre d'état ;
- le renommage d'un symbole écrit par remplacement les fichiers visés, et
  l'histoire d'annulation du document ouvert repart avec ;
- l'aller à la définition d'une citation ne mène nulle part : la
  bibliographie n'est pas un fichier éditable ici ;
- tableaux et formules mathématiques passent encore par le bloc Typst
  brut ; leur syntaxe propre vient ensuite.

## Licence

MIT — voir `LICENSE`.
