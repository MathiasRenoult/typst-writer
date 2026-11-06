# Le `.tw` transportable — spécification du format (v1)

Demande 3 de `bug-report-7.md` : « un fichier `.tw` unique soit
suffisant pour effectuer le transfert ». Premier jalon : **le format**.
Implémentation : `src/langage/portable.rs` (`FichierPortable`),
éprouvée par quatre tests de va-et-vient.

## La structure

Le document reste ce qu'il est : le texte que l'éditeur montre, en tête
du fichier, inchangé pour qui n'utilise pas le transportable. Les pièces
embarquées suivent, dans une **clôture** :

```text
= Mon document

Du texte, comme toujours.

~~~~ tw-portable:1
@meta
titre: Ma thèse
auteur: Alice
@normes
<le TOML de mise-en-page.toml, octet pour octet>
@bibliographie
@book{dupont2023,
  title = {Un titre},
}
~~~~
```

- `~~~~ tw-portable:N` ouvre la clôture, `N` est la version du format ;
- une ligne `@nom` ouvre une section — seuls `@meta`, `@normes` et
  `@bibliographie` sont des noms de section ; un `@book{…}` en
  bibliographie est une **notice**, pas un changement de rubrique ;
- `~~~~` seul ferme.

## Les pièces

| Section | Contenu | Reconstruit |
|---|---|---|
| `@meta` | paires `clé: valeur` | `metadonnees.toml`, les entrées `sys.inputs` |
| `@normes` | le TOML des normes | `mise-en-page.toml` (et le gabarit, par `appliquer`) |
| `@bibliographie` | le BibTeX | `references.bib` |

Ce que le rendu compose en plus (page de titre, sommaire, en-têtes) naît
des **normes + métadonnées** — pas d'une quatrième pièce : ce qui se lit
dans le `.tw` suffit à tout reconstruire.

## La rétrocompatibilité

- Un `.tw` **sans clôture** est un `.tw` d'avant : le texte entier est le
  document, aucune pièce, rien ne bouge (`un_fichier_d_avant_lit_sans_`).
- La version est lue : un fichier écrit par une version **future** se
  lit, mais l'app refuse d'écrire par-dessus (`version_inconnue`) — les
  pièces qu'elle ne voit pas ne se perdent pas. C'est la règle de la
  maison : une donnée ne se perd jamais.

## Les jalons suivants

1. **L'écriture** : « Exporter le projet » propose le `.tw` unique
   (les pièces viennent du projet ouvert).
2. **La lecture** : ouvrir un `.tw` portant une clôture propose de
   reconstruire le projet (pièces écrites à leur place, `appliquer` pour
   l'accroche, puis ouverture).
3. **L'éditeur** : la clôture se replie (le mécanisme de repli du
   vendor) — le document se lit sans sa galerie technique.
