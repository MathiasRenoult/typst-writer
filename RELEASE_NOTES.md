# Typst Writer 0.6.2

**Gérer ses versions bibliques, et voir les erreurs sans quitter la page.**

Cette version fait du panneau biblique un vrai espace de gestion : les
versions ajoutées montrent leurs livres, leurs chapitres et leur provenance,
se choisissent, s'exportent pour être transportées et se suppriment vers la
corbeille. Les références déjà ouvertes ne changent plus de version quand le
registre bouge, et le nom d'un livre suit la version qui le nomme.

## Nouveautés de la 0.6.2

- Liste des versions ajoutées avec nombre de livres et de chapitres,
  provenance (`source`), ouverture du dossier, export du JSON, modèle JSON
  minimal et suppression réversible.
- Un écran **Restaurer une version** réunit les copies de `sauvegardes/` :
  date locale, taille, empreinte, aperçu, confirmation avant remplacement ; le
  texte actuel est sauvegardé avant d'être remplacé, et le document affiché
  est réellement rechargé.
- Indices stables des versions pendant la session : ajouter ou retirer une
  version ne décale plus une référence ouverte ; supprimer la version
  consultée revient à Louis Segond.
- Noms de livres dépendants de la version citée : une version qui nomme
  « Gen » autrement affiche son propre nom.
- Convertisseur `scripts/convertir-version-biblique.py` pour USFM simple et
  JSONL de versets, avec conservation des trous de numérotation.
- Les erreurs de compilation sont soulignées dans l'éditeur et listées dans le
  panneau gauche en écriture ; un clic ouvre la position fautive dans la
  source.
- L'export DOCX rappelle avant le choix du fichier que les notes deviennent
  du texte ; le PDF reste la sortie finale recommandée.
- L'export d'archive propose de joindre les versions bibliques ajoutées :
  avertissement sur les droits, empreinte dans le manifeste et restauration à
  l'import. L'export JSON séparé reste disponible.
- La recherche peut parcourir tout le projet : chemin, ligne et extrait pour
  chaque occurrence, clic pour l'ouvrir ; le remplacement reste local au
  document courant. Elle attend la fin de la frappe et abandonne un parcours
  devenu inutile.
- L'index des citations ne confond plus deux traductions d'un même livre, et
  les archives portables portent une empreinte stable d'une exécution à
  l'autre. Ses divisions de corpus sont des titres de niveau 2 ; un titre de
  livre ne reste jamais seul en bas d'une colonne.
- La recherche du document est une seule barre, non modale : on continue à
  parcourir et à cliquer dans l'éditeur, et « suivant » / « précédent »
  montrent l'occurrence dans la fenêtre. En rendu, `Ctrl+F` ouvre la recherche
  du PDF, jamais les deux à la fois.
- Le passage édition ↔ rendu garde l'endroit de lecture : l'édition revient à
  son défilement exact, le rendu à sa position exacte ; la première ouverture
  du rendu suit le haut de la page d'édition.
- La **palette de commandes** réunit maintenant les formats du texte (gras,
  italique, souligné, barré, code, formules, titres, listes, citation, note),
  les teintes orange, verte et la dernière couleur choisie, la roue des
  couleurs, tous les imports et exports, les thèmes, les normes de la
  bibliothèque et l'activation des plugins.
- Les liens internes peuvent afficher la numérotation réelle :
  `[[titre-adoption-coutume|"chapitre"]]` devient « chapitre 2.3 »,
  `[[figure-planche|"figure"]]` devient « Figure 4 », et
  `[[titre-adoption-coutume|"page"]]` devient la page composée. La forme
  courte `adoption-coutume` suffit quand elle est unique.
- Les **pages de garde** se délimitent par `{{garde}}` … `{{/garde}}` : elles
  passent avant le sommaire, sans numéro et sans entrée dans la table des
  matières, même avec des titres. Plusieurs blocs font plusieurs pages ; le
  menu `..` les insère. La page de titre des normes est aussi sans numéro.
- Les citations collées au mot ou placées en fin de paragraphe restent des
  citations Typst : la phrase garde ses espaces, le guillemet fermant son
  insécable, et une adresse électronique reste du texte.
- Les numéros de ligne s'activent ou se désactivent dans la barre d'outils, et
  la préférence est retenue d'une session à l'autre.
- **Aide → Corbeille et suppressions…** montre ce qui a été supprimé, avec sa
  date et son origine, et le restaure sans écraser un fichier existant.
- L'aperçu est plus direct : le PDF est exporté depuis le document compilé
  sans seconde compilation, les pages sont centrées, les erreurs prennent
  toute la largeur du panneau et aucun texte de compilation ne s'affiche
  au-dessus du document. Une bande de la page voisine est préchargée pour
  éviter les pages blanches au défilement rapide ; les pages visibles passent
  avant le préchargement, et une ancienne vue n'occupe plus les places de
  rasterisation.

## Nouveautés de la 0.6.1

Cette version reprend les temps forts de la 0.6.0 et corrige ce qui restait
sur le chemin des balises canoniques : l'index, le clic sur une référence et
la saisie d'un accent. La palette réunit les gestes, les citations s'ouvrent
au fil de la frappe, la Bible gagne ses livres hors canon, et les documents
longs respirent.

- L'index des citations sait séparer les livres canoniques et non canoniques
  selon le canon choisi dans les normes.
- Les titres, sous-titres et figures reçoivent une ancre et deviennent
  linkables par `[[référence|texte]]` ; les liens sans cible sont signalés,
  et un clic les sélectionne dans la source.
- Un document `.typ` peut être importé et traduit en `.tw`, les constructions
  non traduisibles restant en blocs Typst bruts.
- `{{d aujourd'hui}}` et `{{d maintenant}}` composent la date et l'heure de la
  composition ; `SOURCE_DATE_EPOCH` permet une sortie reproductible.
- Des versions bibliques locales peuvent être ajoutées depuis un fichier JSON,
  avec leur propre nombre de livres et leurs textes apocryphes ou
  deutérocanoniques.

## Corrections de la 0.6.0

- L'index des citations bibliques reconnaît les balises `{{b …}}` écrites
  par l'application ; il ne reste plus vide après une insertion.
- Le clic et le survol d'une référence canonique retrouvent le verset et sa
  fiche d'édition.
- La palette ne panique plus si le curseur tombe au milieu d'un caractère
  accentué.
- La batterie de vérification retrouve son binaire de test et couvre les
  167 épreuves applicatives.

## Les temps forts

### Une palette pour tout le document

`Ctrl+K` ou `/` ouvre les commandes ; `..` propose les balises, les blocs et
les modèles de passage. Les couleurs, les dates, les termes du glossaire et
les métadonnées ont désormais des formulaires qui montrent le texte prêt à
être inséré. Le curseur revient dans la phrase après chaque insertion.

### Les citations sans rupture

Taper `@` ouvre le composeur de citation. On choisit la source, la page,
puis une autre référence si besoin ; une citation existante se rouvre au
curseur. Les références viennent de la bibliographie du projet, d'un fichier
`.bib` ou de Zotero, et les styles de citation Zotero sont reconnus.

La bibliographie se gère d'un geste : **Tout supprimer** vide
la bibliothèque locale — une copie complète part à la corbeille —, et une
collection Zotero liée peut être changée ou déliée depuis sa ligne. Les
crayons et corbeilles des entrées restent visibles.

### Les chapitres, sans quitter le document

Une tête de niveau 1 ouvre un chapitre. Le panneau gauche propose **Document
entier** ou un chapitre ; la vue par chapitre ne montre que ses pages, et le
sommaire se réduit à ses têtes. La source reste entière dans l'éditeur :
choisir un chapitre ne réécrit jamais `document.tw`.

### La Bible jusqu'aux marges du canon

Le panneau biblique est une liste de livres cherchable : quelques lettres
suffisent, et l'on insère sans passer par **Ajouter**. Les abréviations de la
TOB sont disponibles, et les deutérocanoniques comme les apocryphes — Tobie,
Judith, Sagesse, Hénoch, Jubilés, Testaments des patriarches, apocalypses —
se citent et s'indexent. L'index donne les pages réelles du PDF.

### Lire le document composé

Le lecteur de PDF permet de sélectionner et copier le texte final, de
chercher une expression, de suivre les liens et de parcourir les pages. Le
rendu accompagne l'écriture sans la bloquer.

### Un espace de travail durable

Panneaux et onglets, plusieurs vues du même chapitre, reprise de la
disposition au lancement, bibliothèque de projets avec couvertures et
vignettes. Les sauvegardes automatiques, les copies précédentes et le verrou
de projet protègent le travail ; les projets s'exportent en archive portable
ou en fichier `.tw`.

## Télécharger et installer

Le paquet Linux est fourni avec son empreinte :

    typst-writer-0.6.2-linux-x86_64.tar.gz
    typst-writer-0.6.2-linux-x86_64.sha256

Installation sans `sudo` :

    tar -xzf typst-writer-0.6.2-linux-x86_64.tar.gz
    cd typst-writer-0.6.2-linux-x86_64
    ./install.sh

Vérification facultative du paquet :

    sha256sum --check typst-writer-0.6.2-linux-x86_64.sha256

La désinstallation se fait avec `./desinstaller.sh` et ne touche jamais aux
documents. Les mises à jour se font avec le paquet choisi : l'application ne
télécharge rien à votre insu.

## À propos de cette version

Aucune donnée n'est envoyée : les projets, les bibliographies, les normes et
les sauvegardes restent locaux. Les limites connues — export DOCX par
LibreOffice avec les notes en texte, import DOCX absent, Zotero en lecture
seule — sont décrites dans le
[README](https://github.com/MathiasRenoult/typst-writer/blob/master/README.md).

## Documentation

- [Prise en main](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/prise-en-main.md)
- [Écrire](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/ecrire.md)
- [Bibliographie](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/bibliographie.md)
- [Normes et mise en page](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/normes.md)
- [Raccourcis](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/raccourcis.md)

Bonne écriture.
