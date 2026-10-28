# Rechercher et remplacer

`Ctrl+F` ouvre la barre de recherche ; `Échap` la ferme. Elle flotte sur
le texte, et le reste de l'éditeur continue de vivre derrière elle.

## Le motif est une expression régulière

Chercher, c'est écrire une expression régulière — `chapitre \d+`, `^# `,
`\bthéologie\b`. Les deux champs — le motif et le remplacement — sont
conduits au clavier :

- `Entrée` dans le motif cherche le **suivant** ;
- `Entrée` dans le remplacement **remplace l'occurrence courante** ;
- `F3` et `Maj+F3` vont au suivant et au précédent, même la barre fermée
  — `F3` la rouvre au besoin.

Le compte — `3/17` — suit ce qu'on visite.

## Le remplacement rappelle ses groupes

Un motif peut **capturer** : `(\w+)@(\w+)` prend deux groupes. Le
remplacement les rappelle au dollar : `$1-$2`. Tout se fait du dernier au
premier — **Tout remplacer** ne décale jamais les occurrences restantes.

Un motif fautif se dit dans la barre, en rouge, sans jamais effacer ce
qui est cherché.
