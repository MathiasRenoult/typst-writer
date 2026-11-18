# Ajouter une version biblique

L'application embarque Louis Segond 1910 et J.N. Darby. Une version
supplémentaire s'ajoute **localement**, à partir d'un fichier JSON que vous
fournissez : l'application ne télécharge aucun texte et n'en distribue aucun.
N'installez donc qu'une version que vous avez le droit d'utiliser.

## Convertir un fichier courant

Le script `scripts/convertir-version-biblique.py` traduit vers ce format un
JSONL de versets ou un fichier USFM simple (`\id`, `\h`, `\c`, `\v`) :

    python3 scripts/convertir-version-biblique.py \
        --entree ma-bible.usfm --id vulgate --nom "Vulgate" \
        --source "Texte que je possède" --sortie vulgate.json

Le JSONL accepte une ligne par verset, avec `osis`, `nom`, `abreviation`,
`chapitre`, `verset` et `texte`. Les numéros absents deviennent des chaînes
vides : la numérotation de l'édition reste intacte. Le script ne télécharge
rien et refuse les identifiants réservés ou les livres en double. Un USFM
complet contient souvent des notes et des titres de section : le convertisseur
les ignore et ne conserve que le texte des versets.

## Gérer les versions installées

La liste **Versions ajoutées** apparaît sous les boutons de tri dès qu'une
version existe. Elle indique le nombre de livres de chaque version.

- Cliquer sur un nom choisit cette version pour la consultation.
- Le nombre de livres et de chapitres, puis la provenance (`source`) déclarée
  par le fichier, restent visibles sous le nom.
- Le bouton **modèle** (à côté du `+`) enregistre un JSON minimal valide,
  qu'il suffit de remplir et de réinstaller avec le `+`.
- **Dossier** ouvre `~/.typst-writer/bibles/` dans le gestionnaire de fichiers.
- La flèche de téléchargement copie le fichier JSON à l'endroit choisi.
- À l'export d'une archive de projet, une question propose de joindre les
  versions ajoutées. **Oui** les copie dans l'archive avec leur empreinte et
  les restaure à l'import ; **Non** laisse l'archive au projet seul. Dans les
  deux cas, l'export JSON séparé reste disponible pour transporter un fichier
  isolé. À l'import d'une archive qui contient des versions, une confirmation
  rappelle que vous devez détenir les droits sur ces textes.
- La corbeille retire la version : son fichier part à la corbeille du système,
  il n'est pas effacé. La consultation revient à Louis Segond si c'était la
  version supprimée.

Les références déjà ouvertes ne sont pas déplacées quand on ajoute ou retire
une autre version : les versions ajoutées reçoivent un emplacement stable pour
la durée de la session, et une version remplacée par un fichier de même
identifiant garde le sien.

## Le geste

Dans le panneau de gauche, ouvrez **Statistiques bibliques**, puis cliquez sur
le **+** à côté des boutons de tri. Une confirmation rappelle que vous devez
détenir les droits sur la version, puis vous choisissez le fichier JSON.
Elle est copiée dans `~/.typst-writer/bibles/` et apparaît aussitôt dans la
liste **Version** du panneau d'insertion, dans **Traduction consultée** de
la référence cliquée et dans les normes du document.

Un fichier portant le même identifiant remplace la version précédente. Un
fichier refusé (JSON invalide, doublon de livre, identifiant réservé) n'écrit
rien ; le message dit pourquoi.

## Le fichier

```json
{
  "id": "vulgate",
  "nom": "Vulgate",
  "langue": "la",
  "source": "Texte que je possède et que j'ai le droit d'utiliser",
  "livres": [
    {
      "osis": "Gen",
      "nom": "Genesis",
      "abreviation": "Gn",
      "chapitres": [
        ["In principio creavit Deus caelum et terram.", "Terra autem erat…"]
      ]
    },
    {
      "osis": "Tob",
      "nom": "Tobias",
      "abreviation": "Tb",
      "chapitres": [
        ["Premier verset.", "Deuxième verset."]
      ]
    }
  ]
}
```

Les règles sont volontairement simples :

- `id` est le code écrit après la barre verticale d'une citation :
  `{{Tob 1.1|vulgate}}`. Il est unique, en minuscules, et ne peut pas être
  `lsg` ni `darby`.
- `livres` contient **exactement** les livres de cette version, dans l'ordre
  qu'elle donne. Une version de vingt livres, ou qui ajoute Tobie, Judith et
  les Maccabées, est donc normale.
- chaque `osis` est unique dans le fichier. Employez les codes que
  l'application connaît déjà (`Gen`, `Tob`, `1En`, `Jub`…), afin que les
  citations et l'index retrouvent le livre sans ambiguïté. Un livre propre
  à votre version peut aussi recevoir un code et un nom uniques : le nom
  devient citable si aucune autre version ne l'emploie pour un autre code.
  Le nom et l'abréviation sont ceux que vous voulez voir dans les listes.
- `chapitres` se lit dans l'ordre : le premier tableau est le chapitre 1,
  son premier élément le verset 1, ainsi de suite. Une chaîne vide conserve
  le numéro du verset et n'affiche rien.

Le champ `source` est libre ; il sert à noter la provenance et la base
juridique de votre exemplaire. Il reste dans le fichier local et n'est pas
publié par l'application.

## Ce que la version change — et ce qu'elle ne change pas

La version choisie fournit le **texte consulté** (bulle, référence cliquée,
statistiques) et la liste des livres du panneau d'insertion. Elle ne change
pas les citations déjà écrites : une citation sans suffixe suit la version
consultée, une citation avec suffixe garde la sienne.

Les formes d'affichage des livres (`|long`, `|court`, `|tob`…) et les normes
de mise en page restent celles du document. Les textes ajoutés ne sont pas
inclus dans une sauvegarde de projet : ils vivent dans les données locales de
l'application.
