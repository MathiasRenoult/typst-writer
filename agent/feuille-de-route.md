# Feuille de route — de la version 0.2 à la beta

Trois arbitrages la fondent, posés le 28 octobre 2026 :

- **la beta s'adresse à des thésards et écrivains volontaires**, quelques-
  uns : la donnée est sacrée, la doc tient debout, mais rien n'exige un
  paquetage grand public ;
- **la syntaxe `.tw` portera les tableaux et les formules** avant la beta
  — ce sont les deux absences que le bloc brut ne comble pas pour écrire
  une thèse ;
- **le site de documentation vient avec la beta** : le manuel est déjà en
  `.md`, prêt à être rendu.

## La méthode

Chaque étape est une **version alpha publiée** — `v0.3.0-alpha.N`, en
pré-version sur le dépôt — et non une promesse. Une étape n'est close que
quand :

1. ses correctifs sont **mesurés** : l'épreuve qui les crie est écrite
   avant la correction, et verte après ;
2. les tests sont verts, clippy à zéro, aucun reliquat silencieux ;
3. ses notes de version ne disent que ce qui a été vu.

L'ordre va de ce qui peut nuire à qui nous confie ses textes, vers ce qui
manque à son travail, vers ce qui fait bonne figure.

---

## α.1 — La parole donnée (la donnée) ✅

**Fait** — les six pièces, éprouvées : le test de thèmes écrit en dossier
temporaire, le trousseau stabilise les clés (et deux notices ne partagent
jamais la leur), les fichiers corrompus se disent (session, liaison,
projet), les suppressions demandent leur mot, le verrou d'instance ferme
l'écriture à qui ne le tient pas.

## α.2 — Les tableaux ✅

**Fait** — la forme markdown, ses alignements, les cellules en texte
inline, la barre échappée, le décor hérité, l'édition par blocs, et le
round-trip éprouvé au hasard. Une ligne sans séparation reste du texte.

## α.3 — Les formules ✅

**Fait** — `$…$` en ligne et `$$…$$` en bloc : des bandes opaques comme
le code, où rien ne s'ouvre ni ne se ferme. Les deux formes traversent le
round-trip, et une formule mal fermée reste du texte — jamais d'erreur,
jamais de blocage.

## α.4 — Le quotidien sans accroc ✅

**Fait** — les projets se gèrent (création, renommage, suppression vers
la corbeille), la session rouvre ce qui était ouvert, les culs-de-sac se
ferment, les redites s'effacent, ce qui se tait se dit, la navigation au
clavier des listes est complète.

## α.5 — Le seuil de la beta ✅

**Fait** — la mesure, le site, et le gel.

- **La thèse à l'épreuve.** Un document de synthèse de **250 pages**
  (40 chapitres, notes comprises) compile en **0,56 seconde** au vrai
  Typst.
- **Le site de documentation.** `python3 site/construire.py` — sans
  aucune dépendance — rend le manuel en `site/dist/`.
- **Le gel des fonctions** est pris : ce qui entre en beta est ce qui
  est ici.

---

## Ce qui est entré en plus (0.3.0-alpha)

Au-delà de la feuille de route, la version 0.3.0-alpha porte :

- **l'espace de travail** : panneaux divisibles, onglets, vues partagées,
  disposition restaurée ;
- **la bibliothèque plein écran** : galerie de couvertures, étagères ;
- **les graphiques** dans la langue (`{graphique}` avec cetz) ;
- **l'overlay de note de bas de page** (clic sur `^[]`) ;
- **l'éditeur de thème** complet ;
- **les trois rapports de bugs soldés** (18 + 14 + 17 points) ;
- **la qualité** : `deny.toml`, zéro `unsafe`, fuite mémoire corrigée,
  `Eq`, `const fn`, perfs.

## La beta — v0.3.0-beta.1

Elle s'ouvre à quelques volontaires. Le contrat de cette phase :

- **plus de fonctions nouvelles** : seuls les correctifs entrent, et les
  retours des volontaires les commandent ;
- **la donnée reste sacrée** : tout rapport de perte de texte est traité
  avant tout autre signalement ;
- les notes de version suivent l'alpha dont le correctif vient.

Après la beta : les retours façonnent `v0.3.0`, puis `v1.0`.

## Ce qui n'attend pas la beta

Il est noté pour être dit honnêtement, pas caché : les formes étendues
du décor (formes libres, ombres), l'image en ligne, les styles nommés des
normes, et les promesses de performance au-delà de la thèse mesurée. Ce
sont des suites, pas des manques de ce palier.
