# Le glossaire

Un glossaire porte les termes du document — sigles, concepts, noms
propres — avec leurs formes et leur définition. Il se gère entièrement
du panneau gauche, et le paquet [glossarium](https://typst.app/universe/package/glossarium)
s'occupe du rendu : première occurrence développée, reprises serrées,
liste finale avec renvoi aux pages.

## Gérer les termes

La section **Glossaire** du panneau gauche, sous la bibliographie :

- **Nouveau terme** (l'icône `+`) ouvre la fiche : clé, forme courte
  (sigle), forme longue, définition, formes plurielles, groupe.
- Un **clic sur un terme** de la liste insère sa référence au curseur.
- Au survol, le **crayon** corrige le terme, la **corbeille** le
  supprime.
- La **loupe** cherche un terme — par sa clé, ses formes ou sa
  définition.

Les termes vivent dans `glossaire.toml`, à la racine du projet, dans
l'ordre où vous les avez posés. Ils voyagent avec le projet — archive
portable et `.tw` unique.

**La clé est un identifiant** : lettres, chiffres, `-`, `_`, `.` — sans
espace ni deux-points. Elle ne bouge plus une fois citée : pour une
autre clé, créez un terme neuf et supprimez l'ancien.

## Référencer un terme

Dans le texte :

| Ce que vous écrivez | Ce que le document rend |
| --- | --- |
| `{glo:sdo}` | la forme du terme, développée à la première occurrence |
| `{glo:sdo pluriel}` | la forme plurielle |
| `{glo:sdo majuscule}` | la même, avec majuscule — en début de phrase |
| `{glo:sdo majuscule pluriel}` | la majuscule et le pluriel |

Sur un chapitre `.typ` brut, l'insertion au curseur écrit la forme
Typst équivalente : `#gls("sdo")`.

## La liste finale

Là où vous la voulez — en général à la fin :

```
{glossaire}
```

Par défaut, la liste ne montre que les termes **référencés** dans le
document. `{glossaire: tout}` y ajoute ceux que vous avez définis sans
les citer.

Sans `{glossaire}`, les termes se composent sans lien : il n'y a pas de
liste où pointer. Un terme dont la clé n'existe pas se compose en
`[clé]` et se signale dans les diagnostics — le document compile
toujours.

## Les formes

À la première occurrence, le terme développe sa **forme longue**, et la
reprise garde la **forme courte**. Dans la fiche :

- *Forme courte* — le sigle : « sdo » ;
- *Forme longue* — le développement : « docteur scolastique » ;
- *Pluriels* — les formes plurielles ; sans elles, glossarium
  improvise ;
- *Définition* — la glose, dans la liste finale ;
- *Groupe* — la rubrique de la liste finale.
