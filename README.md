# Typst Writer

Un éditeur pour écrire des documents en Typst — thèses, mémoires, livres.
L'aperçu affiché est celui du document final : c'est le vrai compilateur
Typst qui le compose.

**Version 0.3.0-alpha.3 — pré-version.** Utilisable au quotidien — c'est
avec elle qu'elle s'écrit — mais elle change encore. Elle porte R1 à R8
de la feuille de route : le coffre-fort des documents (sauvegardes,
verrou, écritures sûres), les imports sans perte, les références fidèles,
les styles bibliographiques, le glossaire, les modèles de documents, les
exports et le paquet Linux. Le détail est dans le
[CHANGELOG](CHANGELOG.md).

## Ce qu'elle fait

- **Écrire** dans une couche markdown lisible, `document.tw` — titres,
  emphase (gras, italique, souligné, barré), citations, notes, liens,
  images, tableaux, formules, graphiques, décor, commentaires, sauts de
  page — qui se compare ligne à ligne, et dont le Typst reste le maître
  du rendu ;
- **Composer** dès que l'écriture s'arrête : la page se montre telle que
  le lecteur la verra, et l'aperçu suit le chapitre sous le curseur ;
- **Travailler en espace** : panneaux divisibles, onglets par panneau,
  même document dans plusieurs vues (édition / rendu / Typst), disposition
  restaurée au lancement ;
- **Citer** : une bibliographie par projet, une fiche complète (éditeur,
  lieu, titre court, type de document), un projet lié à une collection
  **Zotero** — lue, jamais réécrite — et les **styles bibliographiques
  CSL** : catalogue du moteur, styles du projet, styles Zotero, import de
  `.csl` et de `.bib` ;
- **Mettre en page** par des normes — papier, marges, titres, couleurs,
  typographie par langue, décor, en-tête et pied de page ;
- **Chercher** en texte littéral ou en expressions régulières, avec
  remplacement à rappel de groupes ; la mise en évidence et le remplacement
  comportent encore des limites décrites dans le guide ;
- **Personnaliser** l'apparence : onze thèmes livrés, un éditeur de thème
  intégré, des étagères pour classer les projets ;
- **Structurer** un document par les modèles livrés — classique, devoir
  court, travail long — avec notes de bas de page, renvois et figures
  résolus par le vrai compilateur ;
- **Exporter** en PDF, en Markdown (fidèle) et en DOCX (via LibreOffice,
  les notes y devenant du texte — la limite est dite dans l'application),
  et **faire voyager un projet** : une archive portable avec l'inventaire
  de ce qui manque, et un import qui n'écrase jamais sans choix, récupère
  un import interrompu, et garde ce qui est remplacé dans la corbeille ;
- **Travailler au clavier d'un bout à l'autre** — `F1` montre les
  raccourcis, les premiers pas, le **diagnostic** du poste et la
  **récupération** (corbeille, sauvegardes, temporaires perdus).

## Écrire

Le document se compose en markdown :

    # Le titre

    Un paragraphe, avec du **gras** et une citation — @dupont[p. 45].

    | Mois | Ventes |
    | --- | :-: |
    | Jan | 10 |

    Soit $x^2$ dans le texte.

    {graphique type=barre titre="Évolution"}
    Jan | 10
    Fév | 20

Les mesures de mise en page vivent dans les normes du document ; la
source dit l'intention. La syntaxe entière est dans
[la documentation](docs/guide/ecrire.md).

## L'espace de travail

L'éditeur se scinde en panneaux (glisser un bord), chacun avec ses
onglets. Le même document peut s'ouvrir dans plusieurs vues — texte
partagé, aperçus indépendants. La disposition se retrouve au lancement.
La bibliothèque de projets se parcourt en galerie plein écran, avec
couvertures et étagères.

## Installation

Sous Linux, le paquet `typst-writer-<version>-linux-x86_64.tar.gz` s'installe
d'un geste, sans sudo : `./install.sh` — se met à jour en se réexécutant, se
retire par `./desinstaller.sh`, chaque fichier vérifié par son empreinte.
Voir [l'installation Linux](docs/guide/installation-linux.md). Il faut un
Linux avec Wayland ou X11, et un pilote graphique offrant Vulkan.

Ou, avec une chaîne Rust à jour :

```sh
cargo build --release
./target/release/typst-writer
```

Sous Windows et macOS, les chemins des données et les ouvertures sont
prévus (voir le guide des fichiers) ; les paquets natifs restent à
construire et à éprouver sur leurs machines.

L'assistance à la frappe s'appuie sur
[tinymist](https://github.com/Myriad-Dreamin/tinymist), qui est
**facultatif** : sans lui, tout le reste fonctionne.

## Documentation

Le manuel d'usage est dans [`docs/guide/`](docs/guide/index.md) — prise
en main, écriture, bibliographie, normes, recherche, raccourcis, fichiers.

Il devient un site d'une commande, sans dépendance :

```sh
python3 site/construire.py
```

Le site se lit dans `site/dist/`.

## Qualité et précautions

La suite de tests ne couvre pas tous les gestes de l'interface. La sauvegarde
automatique est différée : conservez des copies de vos travaux importants et
vérifiez les références ainsi que le PDF avant de remettre un document.

## Limites connues

- les diagnostics du compilateur ne sont pas soulignés dans le texte :
  ils sont comptés dans la barre d'état ;
- le renommage d'un symbole écrit par remplacement les fichiers visés, et
  l'histoire d'annulation du document ouvert repart avec ;
- l'aller à la définition d'une citation ne mène nulle part : la
  bibliographie n'est pas un fichier éditable ici ;
- la connexion Zotero et le rendu des citations doivent être vérifiés dans
  le PDF ; voir les réserves du [guide bibliographique](docs/guide/bibliographie.md) ;
- les champs dynamiques d'en-tête et de pied (`{projet}`, `{document}`,
  `{date}`, `{auteur}`, `{titre}`, `{page}`, `{pages}`) sont résolus par
  le moteur ; leur aspect dans le PDF reste à vérifier dans l'application ;
- les graphiques utilisent le paquet `cetz` de Typst ; leur disponibilité
  dépend de la résolution de ce paquet, à vérifier dans l'application ;
- l'export DOCX passe par LibreOffice et porte les notes en texte
  `[note: …]`, pas en notes de bas de page Word ; sans LibreOffice, le
  geste dit son remède (installer LibreOffice ou exporter en Markdown) ;
- l'import DOCX n'est pas livré : les notes perdraient leur identité au
  retour — l'alternative est documentée dans le rapport d'expérience.

## Licence

MIT — voir `LICENSE`.
