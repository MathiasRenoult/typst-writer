# Ajouter une version biblique

L'application embarque Louis Segond 1910 et J.N. Darby. Une version
supplémentaire s'ajoute **localement**, à partir d'un fichier JSON que vous
fournissez : l'application ne télécharge aucun texte et n'en distribue aucun.
N'installez donc qu'une version que vous avez le droit d'utiliser.

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
