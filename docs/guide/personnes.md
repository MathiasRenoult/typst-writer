# Les personnes citées

Le plugin des personnes tient le **registre des personnes** d'un projet et
compose leur **index nominum**. Il ne devine jamais un nom dans la prose :
une personne n'entre dans l'index que lorsqu'elle est nommée par une balise
explicite.

## Le registre

Les personnes vivent dans `personnes.toml`, à la racine du projet :

```toml
[[entrees]]
cle = "augustin"
nom = "Augustin d'Hippone"
nom_simple = "Augustin"
alias = "Saint Augustin"
tri = "Augustin"
naissance = "354"
mort = "430"
qualite = "évêque d'Hippone"
variantes = ["saint Augustin"]

[[entrees]]
cle = "aquin"
nom = "Thomas d'Aquin"
nom_simple = "Thomas"
alias = "le Docteur angélique"
tri = "Aquin"
naissance = "1225"
mort = "1274"
qualite = "théologien dominicain"
```

- `cle` est l'identifiant stable, appelé par le texte. Il ne change pas
  quand le nom affiché change.
- `nom` est le nom complet.
- `nom_simple` est la forme de reprise : sans elle, le nom complet sert de
  repli.
- `alias` est le nom de plume ou la forme d'usage : sans lui, le nom simple
  sert de repli.
- `tri` choisit la place alphabétique : `Aquin` classe Thomas d'Aquin à la
  lettre A, sans imposer au nom une forme rigide.
- `naissance` et `mort` sont les deux bornes des dates ; le champ historique
  `dates` reste lu comme texte libre de repli.
- `qualite` est facultative.
- `variantes` sert seulement à la recherche dans la fenêtre du registre ;
  aucune variante n'est reconnue automatiquement dans le texte.

Le fichier est écrit de façon atomique. Un fichier illisible n'est jamais
remplacé par un registre vide : l'application refuse la sauvegarde et le dit.

## Écrire une mention

```text
{{p augustin}}                         règles des normes : Augustin
{{p augustin|simple}}                  nom simple : Augustin
{{p augustin|complet}}                 nom complet : Augustin d'Hippone
{{p augustin|alias}}                   alias : Saint Augustin
{{p augustin|nom+dates}}              Augustin d'Hippone (354-430)
{{p augustin|simple+naissance+mort}}  Augustin 354 430
{{p augustin|auto}}                    premières règles du plugin
{{personne aquin|Thomas}}              texte libre historique
```

La balise ne retient que la **clé** : le nom affiché reste dans le registre et
peut être corrigé sans réécrire les mentions. Sans expression après `|`, les
règles de première mention et de reprise des normes s'appliquent. Leurs
valeurs par défaut sont le nom simple ; si les normes demandent le nom complet
à la première mention, c'est lui qui s'affiche. Une expression peut être :

- une forme nommée : `complet`, `simple` ou `alias` — le nom seul, sans dates ;
- une liste de champs séparés par `+`, `,` ou `;` : `nom`/`nom_complet`, `simple`/`nom_simple`,
  `alias`, `naissance`, `mort`, `dates`, `qualite` ;
- `auto` : la même règle que la balise nue, écrite explicitement ;
- tout autre texte : une forme libre, conservée telle quelle.

Dans une liste de champs, `dates` ajoute les dates formatées entre parenthèses
(`nom+dates`), tandis que `naissance` et `mort` ajoutent les valeurs
brutes. Un champ absent du registre ne produit rien. Le panneau d'insertion
propose les formes courantes et laisse saisir une expression personnalisée.

L'ancienne syntaxe `{{p clé|}}` se relit comme une mention sans expression ; la
source réécrite préfère `{{p clé}}`.

## Poser l'index

```text
{{index-personnes}}
```

Les alias `{{ip}}` et `{{index-nom}}` sont acceptés. L'index se recalcule à
chaque rendu :

- les entrées sont classées selon `tri`, puis selon la clé ;
- le nom de l'index suit la forme choisie (complet, simple ou alias) ;
- les initiales peuvent faire des sections (`A`, `B`…) ;
- les pages sont demandées à Typst, ce qui donne `p. 12, 18` et `12n` pour
  une mention dans une note ;
- sans pages, l'index affiche le nombre de mentions ;
- les dates suivent la forme choisie et la qualité se montre ou se cache
  dans les normes.

## Gérer le registre

La fenêtre **Personnes citées** s'ouvre par la commande
« Ouvrir les personnes citées » de la palette (`Ctrl+K` ou `/`), par la
section du panneau de travail ou par l'onglet **Personnes** du panneau des
plugins. Elle permet de chercher, créer, modifier et supprimer une entrée,
puis d'insérer `{{p clé}}` ou l'index au curseur.

Le raccourci de la balise « personne » ouvre le **panneau d'insertion** :
un champ de recherche filtre la liste du registre, la fiche choisie remplit
tous ses champs (nom complet, nom simple / nom court, alias, naissance, mort, dates
libres, qualité, variantes), et la balise insérée ne contient que la clé.
Un clic droit sur une balise `{{p …}}` rouvre ce panneau avec la clé et
l'expression déjà remplies ; valider remplace la balise au lieu d'en ajouter
une seconde.

## Les réglages

L'onglet **Personnes** du panneau des plugins règle le registre et l'index
dans le fichier de normes du document :

- les formes du nom pour les balises sans expression (et `auto`) :
  première mention, reprises et index (complet, simple ou alias) ;
- la forme des dates de `auto` (naissance et mort, naissance, mort ou texte
  libre) et leur affichage à la première mention seulement ;
- montrer les pages ;
- grouper sous l'initiale ;
- afficher les dates et la qualité ;
- une à quatre colonnes.

Une clé inconnue n'est jamais effacée : la mention reste visible sous sa
forme de secours, et l'index la montre avec sa clé. Corriger le registre
suffit ensuite à lui rendre son nom.
