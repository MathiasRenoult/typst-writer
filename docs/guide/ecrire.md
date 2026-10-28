# Écrire

Le document se compose dans `document.tw` : une syntaxe markdown, lisible
et comparable ligne à ligne. Ce qui suit est l'état actuel de cette
syntaxe.

## La structure

| Ce qu'on écrit | Ce que c'est |
| --- | --- |
| `# Titre`, `## Sous-titre` | un titre, de niveau 1 à 6 |
| une suite de lignes | un paragraphe ; les lignes se joignent par une espace |
| `> texte` | une citation de bloc |
| `- élément` | une liste à puces |
| `1. élément` | une liste numérotée |
| ```` ```typst ```` … ```` ``` ```` | un bloc Typst brut, inséré tel quel |

Une **ligne vide** sépare les blocs. Ce qui n'est pas reconnu reste du
texte — un fichier abîmé ne bloque jamais l'ouverture.

## Le texte

| Ce qu'on écrit | Ce que c'est |
| --- | --- |
| `**texte**` | le gras |
| `*texte*` ou `_texte_` | l'italique |
| `***texte***` | le gras et l'italique |
| `` `code` `` | du code inline — une bande opaque : rien ne s'y ouvre ni ne s'y ferme |
| `[texte](https://…)` | un lien |
| `^[texte]` | une note libre, en bas de page |

## Les citations

`@cle` appelle une référence de la bibliographie ; `@cle[p. 45]` porte la
page. Plusieurs appels d'un geste — `@dupont[p. 45]; @martin` — ne font
jamais qu'**une** note de bas de page, avec les deux mentions.

L'appel est un **glyphe numéroté** — ¹, ² — dans le fil du texte, un
caractère entier : il se déplace, se sélectionne et se supprime d'un seul
geste. **Cliquer dessus** ouvre le composeur de notes ; les flèches
parcourent la note puis les clés, `Échap` abandonne sans rien écrire.

Voir [La bibliographie](bibliographie.md).

## Les images

    ![La légende de la planche](figs/planche.png)

Une image avec légende compose une **figure numérotée** ; sans légende —
`![](planche.png)` — c'est l'image seule. Le chemin est relatif au projet.

## Les formules

En ligne, entre dollars : `Soit $x^2$ dans le texte.` — une bande
opaque comme le code : rien ne s'y ouvre ni ne s'y ferme.

À part, sur ses lignes :

    $$
    x^2 + y^2 = z^2
    $$

Le contenu est du **math Typst**, mot pour mot. Une barre de dollars
dans le texte s'échappe : `\$`.

## Les tableaux

La forme markdown, avec sa ligne de séparation qui dit l'alignement :

    | En-tête | Valeur |
    | --- | :-: |
    | a | 1 |
    | b | 2 |

`---` aligne à gauche, `:-:` au centre, `---:` à droite. Les cellules
portent du texte ordinaire — emphase, code, citations ; une barre dedans
s'échappe : `\|`. Sans la ligne de séparation, ce n'est pas un tableau :
la ligne reste du texte.

Le décor l'habille comme tout bloc — `{encadré}` prend sa tenue dans les
normes.

## Le décor : l'intention, jamais la mesure

**La source dit l'intention, les normes disent la mesure.** Par défaut,
un bloc paraît comme les normes du document le disent — rien à écrire.

Quand un bloc doit paraître autrement, un **nom** le précède :

    {encadré}
    > une note encadrée

`encadré` prend dans les normes sa couleur de bordure, son épaisseur, son
rayon et sa marge ; `plein` y prend son remplissage. Les noms se
combinent — `{encadré plein}`. La mesure reste possible pour l'exception
— `{encadré rayon=0pt}` — et elle gagne sur les normes. Voir
[Les normes](normes.md).

## L'échappement

Une barre oblique échappe le caractère qui suit : `\*`, `\@`, `\{`
restent du texte, sans aucune interprétation — ni syntaxique, ni
typographique. C'est la porte de sortie universelle : écrire `\;` en
français donne un point-virgule nu, sans l'espace insécable que la langue
y mettrait.

## Ce que la couche ne dit pas encore

Tableaux, formules mathématiques, Typst à la main : le **bloc brut** est
la réponse, aujourd'hui comme demain —

    ```typst
    #figure(
      table(columns: 2, [a], [b]),
      caption: [Un tableau],
    )
    ```

Ce qui s'y trouve va dans le document compilé, mot pour mot. Les formes
propres à la couche — tableaux, formules — viendront avec leur syntaxe
markdown ; la voie leur est ouverte.
