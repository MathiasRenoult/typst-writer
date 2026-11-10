# Rechercher et remplacer

`Ctrl+F` ouvre la barre de recherche ; `Échap` la ferme. Elle flotte sur
le texte, et le reste de l'éditeur continue de vivre derrière elle.

## Deux modes : littéral ou expression régulière

Le bouton `Aa` / `.*` à gauche du motif bascule entre les deux modes :

- **Littéral** (`Aa`) : le motif est pris au pied de la lettre. Un point,
  un dollar, une parenthèse n'ont aucun sens spécial.
- **Expression régulière** (`.*`) : le motif est une regex —
  `chapitre \d+`, `^# `, `\bthéologie\b`.

La barre compte les occurrences trouvées. La mise en évidence dans l'éditeur
utilise un mécanisme distinct et peut ne pas correspondre au motif,
notamment en mode regex : vérifiez la sélection courante.

## Les champs

Les deux champs — le motif et le remplacement — sont conduits au clavier :

- `Entrée` dans le motif cherche le **suivant** ;
- `Entrée` dans le remplacement **remplace l'occurrence courante** ;
- `F3` et `Maj+F3` vont au suivant et au précédent, même la barre fermée
  — `F3` la rouvre au besoin.

Le compte — `3/17` — suit ce qu'on visite.

## Le remplacement rappelle ses groupes

En mode regex, un motif peut **capturer** : `(\w+)@(\w+)` prend deux
groupes. Le remplacement les rappelle au dollar : `$1-$2`. Tout se fait
du dernier au premier — **Tout remplacer** ne décale jamais les
occurrences restantes.

Un motif fautif se dit dans la barre, en rouge, sans jamais effacer ce
qui est cherché.
