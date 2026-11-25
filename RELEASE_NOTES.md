# Typst Writer 0.6.3

**Les personnes citées, une garde nette et des gestes d'édition plus directs.**

Cette version ajoute le registre des personnes citées et son panneau
d'insertion, corrige la page blanche qui précédait une page de garde, et
rapproche les déplacements et suppressions de ce qu'on attend d'un éditeur
moderne.

## Nouveautés de la 0.6.3

- **Personnes citées** : un registre `personnes.toml` par projet, une balise
  `{{p clé}}` et un index `{{index-personnes}}`. Le panneau d'insertion filtre
  le registre au fil de la frappe, remplit la fiche complète et n'écrit que la
  clé dans la balise. La forme du nom et des dates suit les normes ; `|simple`,
  `|complet`, `|alias`, `|auto` et les listes de champs permettent de choisir
  exactement ce qui s'affiche. Un clic droit sur une balise existante la
  rouvre préremplie et la remplace au lieu d'en ajouter une seconde.
- Une balise de personne sans expression applique bien les règles de première
  mention et de reprise. La compilation d'un document qui cite plusieurs fois
  la même personne est couverte par une épreuve de non-régression.
- **Plus de page blanche avant `{{garde}}`** : la bibliographie chargée pour
  les citations ne réserve plus une page, et le branchement de la garde dans
  le gabarit ne se duplique plus.
- **Déplacements par mot** : `Ctrl+flèche` déplace le curseur d'un mot,
  `Ctrl+Maj+flèche` étend la sélection par mot, et `Ctrl+Retour arrière` /
  `Ctrl+Suppr` suppriment le mot précédent ou suivant, accents compris.
- **Menu contextuel au curseur** : le clic droit ouvre ses actions au point
  exact du clic dans la vue de rédaction, sans décalage dû à la barre ou au
  panneau.
- **Menus de la barre du haut** : les cartes « Projet », « Insérer » et
  « Titres » ont une largeur bornée et des lignes neutres ; elles ne prennent
  plus la couleur d'accent ni toute la largeur disponible.
- Un bouton `+` fixe après les onglets ouvre une nouvelle vue sur le document
  courant sans devoir faire défiler la barre d'onglets.
- Le rail des onglets des normes et des plugins défile au lieu d'écraser son
  dernier élément quand la fenêtre est courte.

## Télécharger et installer

Cette version est publiée **sans paquet binaire ni artefact de test**. Le
code source est la référence ; pour l'utiliser, compilez-le avec une chaîne
Rust à jour :

    git clone https://github.com/MathiasRenoult/typst-writer.git
    cd typst-writer
    cargo build --release
    ./target/release/typst-writer

L'application demande Linux avec Wayland ou X11 et un pilote graphique
récent. L'assistance à la frappe est facultative : écrire, composer et
exporter fonctionnent sans elle.

## Ce qui reste inchangé

Les projets, les bibliographies, les normes et les sauvegardes restent
locaux ; aucune donnée n'est envoyée. Le document source reste un fichier
texte `document.tw`, lisible et portable.

Les limites connues sont décrites dans le
[README](https://github.com/MathiasRenoult/typst-writer/blob/master/README.md) :
export DOCX par LibreOffice avec les notes en texte, import DOCX absent et
Zotero en lecture seule.

## Documentation

- [Prise en main](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/prise-en-main.md)
- [Écrire](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/ecrire.md)
- [Bibliographie](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/bibliographie.md)
- [Normes et mise en page](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/normes.md)
- [Raccourcis](https://github.com/MathiasRenoult/typst-writer/blob/master/docs/guide/raccourcis.md)

Bonne écriture.
