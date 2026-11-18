# Typst Writer 0.6.1

**Écrire sans quitter le document — avec les citations bibliques au complet.**

Cette version reprend les temps forts de la 0.6.0 et corrige ce qui restait
sur le chemin des balises canoniques : l'index, le clic sur une référence et
la saisie d'un accent. La palette réunit les gestes, les citations s'ouvrent
au fil de la frappe, la Bible gagne ses livres hors canon, et les documents
longs respirent.

## Nouveautés de la 0.6.1

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

    typst-writer-0.6.1-linux-x86_64.tar.gz
    typst-writer-0.6.1-linux-x86_64.sha256

Installation sans `sudo` :

    tar -xzf typst-writer-0.6.1-linux-x86_64.tar.gz
    cd typst-writer-0.6.1-linux-x86_64
    ./install.sh

Vérification facultative du paquet :

    sha256sum --check typst-writer-0.6.1-linux-x86_64.sha256

La désinstallation se fait avec `./desinstaller.sh` et ne touche jamais aux
documents. Les mises à jour se font avec le paquet choisi : l'application ne
télécharge rien à votre insu.

## À propos de cette version

Aucune donnée n'est envoyée : les projets, les bibliographies, les normes et
les sauvegardes restent locaux. Les limites connues — export DOCX par
LibreOffice, import DOCX absent, diagnostics non soulignés, Zotero en lecture
seule — sont décrites dans le
[README](https://github.com/MathiasRenoult/typst-writer/blob/master/README.md).

## Documentation

- [Prise en main](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/prise-en-main.md)
- [Écrire](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/ecrire.md)
- [Bibliographie](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/bibliographie.md)
- [Normes et mise en page](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/normes.md)
- [Raccourcis](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/raccourcis.md)

Bonne écriture.
