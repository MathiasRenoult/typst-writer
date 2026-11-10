# Typst Writer

Typst Writer est un éditeur de rédaction pour les documents Typst : thèses,
mémoires, livres, articles. L'aperçu est le document final lui-même, composé
par le vrai moteur Typst.

Version 0.5.0.

## Ce que l'application fait

- **Écrire** dans une source lisible et comparable ligne à ligne,
  `document.tw` : titres, emphase, citations, notes, liens, images,
  tableaux, formules, graphiques, décor et sauts de page.
- **Composer** le document pendant l'écriture et suivre le chapitre sous le
  curseur ; le rendu affiché est celui du PDF exporté.
- **Travailler dans un espace de travail** composé de panneaux divisibles,
  d'onglets par panneau et de plusieurs vues d'un même document — édition,
  rendu et source Typst.
- **Citer** avec une bibliographie par projet, des notices complètes, des
  styles CSL, l'import `.bib` et la liaison à une collection Zotero en
  lecture seule.
- **Référencer la Bible** avec une syntaxe dédiée, un panneau d'insertion,
  des formes de livres personnalisables, un gabarit de citation, un index
  paginé et des statistiques de citation.
- **Mettre en page** par des normes : papier, marges, en-tête et pied,
  typographie par langue, titres, couleurs, décor et numérotation.
- **Chercher et remplacer** en texte littéral ou en expression régulière,
  avec aperçu des remplacements.
- **Exporter** en PDF, en Markdown et en DOCX, et transporter un projet sous
  forme d'archive avec inventaire et import sans écrasement.
- **Travailler au clavier** avec une carte des raccourcis (`F1`), un
  diagnostic du poste et la récupération des sauvegardes et de la corbeille.

## Écrire

La source est un markdown lisible :

    # Le titre

    Un paragraphe, avec du **gras** et une citation — @dupont[p. 45].

    | Mois | Ventes |
    | --- | :-: |
    | Jan | 10 |

    Soit $x^2$ dans le texte.

    {graphique type=barre titre="Évolution"}
    Jan | 10
    Fév | 20

Les mesures de mise en page ne sont pas écrites dans la source : elles
vivent dans les normes du document. La syntaxe complète est décrite dans
[le guide d'écriture](docs/guide/ecrire.md).

## Les références bibliques

Une référence s'écrit entre doubles accolades :

    {{Jn 3.16}}

La forme standard est le nom abrégé suivi de `chapitre.verset`. Les formes
historiques `Jn 3:16` et `Jn 3,16` sont également lues. Les livres peuvent
être écrits en entier, en abréviation française ou en code OSIS.

Le bouton **Référence biblique** ouvre un panneau d'insertion : livre,
chapitre, versets (`16`, `16-18`, `16, 17`), plusieurs références et texte
libre sont réunis dans **une seule** balise. Le champ chapitre accepte les
suites de chapitres entiers, par exemple `1-11`.

Les citations peuvent être composées à la main :

    {{2 Ch 5.6 ; Js 1.9 ; 2 Ch 7.6-12}}
    {{Jn 3.16 ; "cf. aussi" ; Gn 1.1}}
    {{Gn 1-11}}

Un même livre n'est nommé qu'une fois : ses occurrences sont séparées par
une virgule. Le rendu peut être rangé dans l'ordre canonique.

### Syntaxe personnalisable

L'onglet **Biblique** des paramètres de plugins règle :

- la forme des livres : abréviation, nom complet, OSIS ou sans point ;
- les chiffres arabes ou romains ;
- les séparateurs entre chapitre et verset, plage, livres, occurrences et
  chapitres entiers ;
- le **gabarit de citation**, qui décrit la forme complète d'une référence.
  Les jetons disponibles sont `{livre}`, `{chapitres}`, `{versets}`,
  `{nombres}`, `{chapitre}`, `{chapitre_fin}`, `{verset}`, `{verset_fin}`
  et `{reference}`. Par exemple, `{livre} {chapitre}, {verset}` rend
  `Jn 3, 16`, et `({reference})` rend `(Jn 3.16)`.

La balise source reste toujours `{{…}}` ; le gabarit règle sa composition.
Une balise peut aussi choisir ponctuellement sa forme avec `|long`,
`|court`, `|osis` ou `|sanspoint`. Une référence peut nommer sa traduction
avec `|lsg` ou `|darby` : elle est alors affichée dans la citation
(`Jn 3.16 (LSG)`), et reste silencieuse sans ce suffixe.

### Index et consultation

Cliquer sur une citation ouvre **La référence cliquée** dans le panneau
gauche. Chaque référence y est présentée séparément avec son livre, son
chapitre et son propre extrait biblique. Le crayon ouvre une fenêtre
flottante pour en corriger le livre, le chapitre ou les versets ; la balise
est réécrite en conservant son texte libre et sa forme.

L'index des citations s'insère par l'icône de la barre supérieure ou par la
balise `{{index}}`. Dans l'onglet Biblique, il peut :

- afficher ou masquer les pages où chaque référence apparaît ;
- regrouper les références sous leur livre ;
- choisir sa propre forme de livre ;
- régler le nombre de colonnes de l'index (1 à 4).

Les pages sont demandées au moteur Typst lors de la composition : ce sont
les pages réelles du document. Une citation placée dans une note est
signalée par le suffixe `n` (`6n`).

Exemple avec regroupement et pages :

    Genèse :
    - 1.18 (p. 2, 5, 6, 18)
    - 3.15 (p. 4, 5)

Les **statistiques bibliques** du panneau gauche listent les références
citées, permettent de les rechercher, de les trier et de rejoindre leur
citation d'un clic. La recherche du panneau prend le clavier à son
ouverture, sans écrire dans l'éditeur.

## L'espace de travail

L'éditeur se divise en panneaux, chacun avec ses onglets. Le même document
peut être ouvert dans plusieurs panneaux avec un texte partagé et des vues
indépendantes. La disposition est retrouvée au lancement. La bibliothèque de
projets se parcourt en galerie plein écran, avec couvertures et étagères.

## Installation

Sous Linux, le paquet `typst-writer-<version>-linux-x86_64.tar.gz`
s'installe sans `sudo` avec `./install.sh`, se met à jour en se réexécutant
et se retire avec `./desinstaller.sh`. Chaque fichier installé est vérifié
par son empreinte. Voir [l'installation Linux](docs/guide/installation-linux.md).

Avec une chaîne Rust à jour :

```sh
cargo build --release
./target/release/typst-writer
```

L'application demande Linux avec Wayland ou X11 et un pilote graphique
offrant Vulkan. L'assistance à la frappe utilise
[tinymist](https://github.com/Myriad-Dreamin/tinymist), qui est facultative :
sans elle, le reste de l'application fonctionne.

## Documentation

Le manuel d'usage se trouve dans [`docs/guide/`](docs/guide/index.md) :
prise en main, écriture et références bibliques, bibliographie, glossaire,
normes et plugins, recherche, raccourcis, fichiers et installation.

Le site de documentation se construit sans dépendance :

```sh
python3 site/construire.py
```

Il se lit dans `site/dist/`.

## Précautions et limites actuelles

La suite de tests couvre le cœur de l'application. Le document est
enregistré automatiquement après une courte pause dans la frappe, et la
version précédente reste dans `sauvegardes/`. Vérifiez néanmoins le PDF
avant de livrer un document et conservez des copies de vos travaux
importants.

- Les diagnostics du compilateur sont comptés dans la barre d'état, mais ne
  sont pas soulignés dans le texte.
- L'histoire d'annulation du document ouvert repart après certains
  remplacements de symboles.
- L'export DOCX passe par LibreOffice et rend les notes comme du texte
  `[note: …]`, et non comme des notes de bas de page Word.
- L'import DOCX n'est pas disponible : les notes perdraient leur identité au
  retour. L'alternative est décrite dans le guide.
- La connexion Zotero et le rendu final des citations se vérifient dans le
  PDF ; voir le [guide bibliographique](docs/guide/bibliographie.md).
- Les graphiques utilisent le paquet Typst `cetz` ; leur rendu dépend de la
  résolution de ce paquet.

## Licence

MIT — voir `LICENSE`.
