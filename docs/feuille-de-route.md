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

## α.1 — La parole donnée (la donnée)

Ce qui peut perdre ou salir un texte part le premier. C'est l'étape que
la beta doit pouvoir garantir par serment.

- **[G33] Le test de thèmes n'écrit plus chez l'utilisateur.**
  `theming::write_example()` cible encore `~/.typst-writer/themes/` :
  `cargo test` crée et supprime un fichier dans les données réelles.
- **[E29] Les clés Zotero stables sont câblées ou leur promesse retire.**
  `zotero/keys.rs` attend son raccordement (`citations.json`) : une
  citation ne doit pas changer de visage quand la collection bouge.
- **[A5] Les fichiers corrompus ne se taisent plus.** `zotero.json`,
  `mise-en-page.toml`, `session.json` corrompus sont dits, et ne sont
  jamais réécrits vides par-dessus leur valeur précédente.
- **[A6] Les suppressions demandent leur chemin.** Corbeille partout
  (fait), mais un geste qui défait un lien ou efface un chapitre se
  confirme — ou s'annule depuis la corbeille.
- **[A7] Un verrou d'instance.** Deux instances sur le même projet
  s'écrasent leurs auto-sauvegardes : un verrou le dit et le refuse.
- **[G34] Les zones muettes reçoivent leurs épreuves** : `fichiers.rs`
  (suppression, import, ouverture), les fichiers corrompus, les thèmes.

**Preuve de l'étape** : une épreuve de perte de texte qui échoue avant et
passe après — y compris fichier corrompu, disque plein, double instance.

## α.2 — Les tableaux

La première pièce de la route du langage (`docs/design/extension-langage.md`),
et la plus attendue d'un écrivain.

- la syntaxe markdown : `| A | B |`, sa ligne de séparation, et
  l'alignement dedans (`:--`, `--:`, `:-:`) ;
- les cellules portent du texte inline — emphase, code, citations ;
- le décor l'habille : `{encadré}`, les mesures aux normes ;
- round-trip prouvé au hasard, édition par blocs, genre de bloc ;
- le manuel : la page d'écriture, et les normes du tableau.

**Preuve de l'étape** : un tableau de thèse — légende, note, citation
dans une cellule — se relit, se réécrit et compose à l'identique.

## α.3 — Les formules

- `$…$` en ligne et `$$…$$` en bloc : des **bandes opaques** comme le
  code, où rien ne s'ouvre ni ne se ferme ;
- les deux formes traversent le round-trip, et une formule mal fermée
  reste du texte — jamais d'erreur, jamais de blocage ;
- [F32] la conception du langage est remise au présent : « dix règles »
  pour onze, `Litteral` et les appels groupés omis.

**Preuve de l'étape** : une page de démonstration — texte, note, tableau,
formule, image décorée — compile au vrai Typst et se relit mot pour mot.

## α.4 — Le quotidien sans accroc

Ce qui piège au fil des jours, et la gestion du projet lui-même.

- **[E31] Les projets se gèrent** : création avec le nom choisi (pas
  `projet-2`), renommage, suppression vers la corbeille, et un dossier
  existant n'est jamais réutilisé sans le dire.
- **[E31] La session rouvre ce qui était ouvert** — pas seulement
  `document.tw` — et « aller à la définition » voit `document.tw`.
- **[D24/D25] Les culs-de-sac se ferment** : le renommage de chapitre
  prend le focus ; « renommer le symbole » sans LSP le dit ; les
  diagnostics d'un autre fichier y mènent ; les fenêtres ouvertes en mode
  focus s'y montrent.
- **[D27] Les redites s'effacent** : le compte de mots trois fois, le nom
  de fichier deux, le titre de fenêtre statique, « Enregistrer sous » qui
  bascule l'éditeur sur un `.typ` et éteint la mise en forme.
- **[B14] Ce qui se tait se dit** : une valeur de typographie inconnue le
  dit au lieu de tomber en `aucune`.
- **La navigation au clavier des listes** : les références et le sommaire
  au clavier, de la première à la dernière ligne.

**Preuve de l'étape** : une session de travail d'une heure — écrire,
citer, renommer, chercher, fermer — sans jamais toucher la souris.

## α.5 — Le seuil de la beta

La tenue à l'échelle d'une thèse, et le seuil qu'on montre.

- **[C20] Le poids à l'épreuve** : un document de thèse mesuré — mémoire,
  compilation, défilement — les trois copies de texte et l'échelle
  minimale revues s'il le faut.
- **Le site de documentation**, rendu depuis `docs/guide/` : le sommaire
  devient la navigation, les captures d'écran y viennent (prises avec
  vous, jamais fabriquées).
- **Le paquet du volontaire** : un binaire qui se décompresse et se lance,
  ses notes d'installation, et les notes de version écrites d'après les
  mesures.
- **Le gel des fonctions** : ce qui entre en beta est ce qui est ici.

**Preuve de l'étape** : la compilation d'une vraie thèse — la vôtre — en
moins de deux secondes, et un premier lecteur extérieur qui installe et
écrit sans aide.

---

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
