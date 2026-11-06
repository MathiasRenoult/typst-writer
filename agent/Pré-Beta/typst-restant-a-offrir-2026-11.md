# Fonctions Typst inaccessibles sans Typst brut — état au 2026-11-02

Ce que Typst sait, et que l'application n'offre encore que par un bloc
brut (`T` dans la barre) ou pas du tout. Chaque ligne dit la voie
d'entrée. Les plus faciles sont choisis et **faits** (voir plus bas).

## Listes et structures

- listes imbriquées (sous-puces) — la couche n'a qu'un niveau ;
- listes de définition (`/ terme : définition`) ;
- tableaux riches (cellules fusionnées, notes de tableau, `table.hline`) ;
- `outline` autre que le sommaire du modèle (figures, tableaux) ;
- parties, annexes, `pagebreak` conditionnel.

## Renvois et notes

- renvois vers des pages (`@clé` ne dit pas la page) ;
- notes de fin (`notes()`), glissantes, d'auteur ;
- `quote` habillée (guillemets de citation avec source).

## Mise en page fine

- colonnes multiples (`columns(n)`) ;
- `place`/`float` (figures flottantes, encadrés posés) ;
- `parbreak`, `h`/`v` réglés, `line` décoratif ;
- numérotation de sections libre (`numbering: "1.1.a"`) — le champ existe
  mais la forme n'est pas proposée dans les normes.

## Mathématiques

- blocs d'équation numérotés et référencés (`$ ... $ <eq>` + `@eq`) ;
- opérateurs personnalisés, `mat`, `vec` guidés.

## Scripts et données

- `json`/`csv`/`yaml` (données pilotant tableaux et graphiques) ;
- `query`/`locate` libres (tout champ dynamique au-delà de `{titre}`) ;
- `state` (compteurs, dépendances entre parties) ;
- `include` conditionnel, `if` documentaire.

## Graphiques

- **cetz** entier : voir la note ci-dessous.

## Choix faits (les plus faciles) — et faits

1. **Quick parts d'en-tête/pied** : `{page}`, `{pages}` existaient ;
   `{document}`, `{date}`, `{auteur}`, `{projet}` existaient en théorie
   mais **rendaient vide** (les `sys.inputs` n'étaient jamais alimentés —
   trou réparé, éprouvé au PDF extrait) ; `{titre}` ajoute (résolu par le
   moteur via `query`) ; les **images** s'y posent (`![…](…)`, à l'échelle
   d'un logo). Les marqueurs morts `\x0N` (défigurés par l'échappement)
   sont réparés au passage.
2. **Choix de couleur** : la roue HSV (teinte, saturation, valeur), centrée
   dans la fenêtre — le choix libre par hex existe désormais sans passer
   par l'éditeur de thème.

## Note : cetz

Le GUI de dessin cetz demande d'abord que le moteur embarqué **résolve
les paquets Typst** (`@preview/cetz`) — l'audit R5 l'a établi
(`cetz_ne_se_charge_pas_dans_le_moteur_embarque`) : le `World` de
`src/typst/render.rs` ne sert aucun paquet. Le chantier, dans l'ordre :

1. servir les paquets depuis un dossier local (paquet vendoré ou cache) —
   `World::file` doit répondre pour les `FileId` de paquet ;
2. alors le GUI devient utile : un composeur de dessin qui écrit du cetz
   dans un bloc brut, avec aperçu réel.

C'est un chantier entier, à traiter à part — pas un supplément de fin de
lot.
