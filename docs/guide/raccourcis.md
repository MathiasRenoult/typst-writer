# Les raccourcis

`F1` ouvre cette même carte dans l'application. Les infobulles de la
barre d'outils portent leurs combinaisons.

## Écrire et le document

| Touches | Effet |
| --- | --- |
| `Ctrl+S` | enregistrer |
| `Ctrl+Z` / `Ctrl+Y` | annuler / refaire |
| `Ctrl+N` | le document d'écriture |
| `Ctrl+O` | charger un `.typ` dans le projet |
| `Ctrl+E` | exporter en PDF |
| `Ctrl+P` | les normes du document |
| `Ctrl+T` / `Ctrl+Maj+T` | le titre suivant / précédent |
| `Ctrl+1` à `Ctrl+6` | insérer un titre de niveau 1 à 6 au curseur |
| `Ctrl+Maj+Haut` / `Bas` | le bloc entier monte ou descend |
| `F2` | renommer le symbole sous le curseur |
| `F12` | aller à la définition |
| `Ctrl+I` | l'assistance à la frappe |
| `Ctrl+Suppr` | supprimer le mot après le curseur |
| `Ctrl+L` | surligner la ligne courante |
| `Ctrl+Maj+L` | surligner la ligne suivante |
| `Tab` / `Maj+Tab` | sur une ligne de liste : l'enfoncer / la remonter d'un niveau |
| `@` | ouvrir le composeur de citation au curseur |
| `Tab` après `@…` | ouvrir aussi le menu contextuel de citation, sans quitter le document |

Un lien interne peut afficher une numérotation calculée : `[[adoption-coutume|"chapitre"]]`
donne « chapitre 2.3 », `[[planche|"figure"]]` donne « Figure 4 » et
`[[adoption-coutume|"page"]]` donne la page du document. Le mot clé doit être
entre guillemets ; sans guillemets, le texte reste un texte ordinaire.

## La palette des commandes

| Touches | Effet |
| --- | --- |
| `Ctrl+K` | ouvrir la palette de toutes les commandes |
| `/` dans le texte | ouvrir la palette système (après un blanc) |
| `..` dans le texte | ouvrir les balises et snippets insérables (après un blanc) |
| `↑` / `↓` | parcourir les résultats |
| `Entrée` | insérer avec les valeurs par défaut |
| `Tab` | ouvrir le formulaire de la balise sélectionnée |
| `Échap` | fermer la palette sans insérer |

La même palette expose les formats du texte — `gras`, `italique`,
`souligné`, `barré`, `gras italique`, `code`, `formule`, `titre 1` à
`titre 6`, listes et citations —, les teintes `orange`, `vert`, `rouge`,
`bleu`, `violet`, `gris`, `noir`, `dernière couleur`, et la roue des
couleurs. On y trouve aussi les imports et exports de fichiers, les thèmes,
les normes de la bibliothèque et l'activation ou la désactivation des
plugins. Tape simplement le nom ou un alias après `Ctrl+K`.

Dans une balise, `Entrée` insère avec les valeurs du formulaire ; `Échap`
annule. Les balises ont des noms courts : `c` (couleur), `g` (glossaire),
`d` (date), `b` (bible), `m` (métadonnée), `e` (encadré), `gr` (graphique),
`t` (tableau), `s` (sommaire), `bib` (bibliographie), `gs` (glossaire de
section), `idx` (index biblique). Les métadonnées du projet apparaissent
dans `..` sous « Métadonnée · clé » et s'écrivent `{{m clé}}` dans le
document.

## Chercher

| Touches | Effet |
| --- | --- |
| `Ctrl+F` | la recherche de l'éditeur, ou celle du rendu quand la vue rendu est active |
| `Ctrl+C` | copier la sélection du rendu |
| `Aa` / `.*` | basculer littéral / expression régulière |
| `F3` / `Maj+F3` | le résultat suivant / précédent |
| `Entrée` | dans le motif : le suivant — dans le remplacement : remplacer |

## L'affichage

| Touches | Effet |
| --- | --- |
| `F11` | le mode focus — la vue active prend toute la fenêtre ; `Échap` en sort |
| `Ctrl+e` | basculer édition / rendu |
| `Ctrl+Maj+e` | basculer en mode Typst (texte généré) |
| `Ctrl+=` / `Ctrl+-` | agrandir / réduire l'aperçu (ou Ctrl+molette) |
| `Ctrl+0` | ajuster l'aperçu à la largeur |
| `Ctrl+Maj+L` | l'aperçu suit le chapitre — ou ne le suit plus |
| bouton des numéros | afficher ou masquer les numéros de ligne de l'éditeur |
| survol | l'extrait d'une référence biblique sous la souris, après un court repos |

## L'espace de travail

| Touches | Effet |
| --- | --- |
| `Ctrl+Maj+N` | scinder le panneau actif verticalement |
| `Ctrl+Maj+O` | scinder le panneau actif horizontalement |
| glisser un onglet | déplacer, détacher ou diviser un panneau |
| `Ctrl+Tab` / `Ctrl+Maj+Tab` | l'onglet suivant / le panneau suivant |
| `Ctrl+Maj+W` | fermer l'onglet actif |
| `Maj+clic` sur un projet | ouvrir dans un nouvel onglet |
| clic sur un projet | remplacer l'onglet actif |

## Les fenêtres

Toutes se conduisent de même.

| Touches | Effet |
| --- | --- |
| `Tab` / `Maj+Tab` | le champ suivant / précédent |
| `Entrée` | valider |
| `Échap` | fermer celle du dessus |

Dans le composeur de citations : les flèches parcourent la note puis les
clés, `Échap` abandonne sans rien écrire. Dans les listes de
complétion : les flèches parcourent, `Entrée` ou `Tab` accepte,
`Ctrl+Espace` accepte aussi, `Échap` ferme.
