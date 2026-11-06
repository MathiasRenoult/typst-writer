# Suivi de développement vers la bêta

Référence : `roadmap-vers-beta-2026-11.md`. Les étapes ne sont validées que lorsque leurs critères automatiques passent ; les essais humains restent à faire par l'auteur. Aucune publication ni aucun push ne sont implicites.

## Historique git (2026-11-02)

Cinq commits, un par jalon, sans push : `7b59047` R2, `d6467a2` R3,
`ce5216a` R4, `0a3bda2` R5, `cd05ad1` R6. Attribution **par fichier** — le
travail s'est entremêlé dans les mêmes fichiers, si bien que les états
intermédiaires de la série ne compilent pas seuls (noté dans chaque
message). L'état final est vérifié : build, clippy `-D warnings`, fmt,
batterie complète verte. Version laissée à `0.3.0-alpha.3` : la
numérotation n'avance qu'à la prochaine release (choix explicite).

## R8 (premier incrément livré) — export portable et import de sauvegarde

- Fait : **`src/project/portable.rs`** — le projet voyage. `inventaire_projet`
  classe ce qui s'y trouve (sources, normes, styles, bibliographies,
  ressources) et **dit ce qui manque** (dépendances appelées mais absentes) ;
  `exporter_projet` écrit une archive unique avec son `manifeste.json`
  (version, nom du projet, inventaire) ; `importer_projet` restitue tout
  **octet pour octet** avec `PolitiqueCollision` : `Refuser` (défaut — la
  moindre collision refuse l'import entier sans rien écrire) ou
  `VersCorbeille` (choix explicite : ce qui est remplacé part à la
  corbeille, réversible). Une cible qui n'est pas un fichier se refuse
  **avant tout déplacement**, et aucun chemin d'archive ne sort de la
  destination (`..`, absolu, lecteur, antislash — le trou jumeau des
  advisories d'archives, verrouillé par `un_import_refuse_un_chemin_qui_sort_du_projet`).
- Fait : **branché dans l'app** — boutons « Exporter le projet (archive)… »
  et « Importer une sauvegarde… » (section Projets), messages
  `ProjetExportPortable`/`ProjetImportSauvegarde`/`ProjetImportChoisi`/
  `ProjetImportConfirmer`, et la **question courte avant le geste** via
  `state.confirmation` (« Remplacer ses fichiers en conflit ? Ils resteront
  dans la corbeille. ») — éprouvée par
  `un_import_de_sauvegarde_se_questionne_avant_de_remplacer`.
- Sécurité : la dépendance `zip` est montée **2.6.1 → 9.2.2** pour purger
  les advisories RUSTSEC-2026-0367/0368/0369 (extraction : bits SUID,
  échappement par liens) — `cargo deny check` vert. API inchangée.
- Défauts trouvés et documentés : la destination d'import est
  `projects_dir()` (et non `work_dir()`) ; un projet neuf n'a pas encore de
  `document.tw` (c'est le premier enregistrement qui l'écrit) — pièges
  d'épreuve, pas de code.
- Fait : **expérimentation DOCX/Markdown mesurée** — rapport livré :
  `agent/Pré-Beta/experience-docx-markdown-2026-11.md` (procédure
  reproductible, matériel, chiffres). La sonde `experience_docx_aller_retour`
  (`src/langage/generer.rs`) rejoue `.tw → HTML → DOCX → HTML` sur une
  fixture aux cinq axes. Relevé (LibreOffice 26.8.1.1) : titres OK,
  tableaux OK (6/6 cellules), citations OK (chaîne intacte), images OK
  (avec légende), **notes transformées** — le texte survit en `[note: …]`,
  sans identité de note de bas de page. Décision du rapport : livrer
  l'**export Markdown** (le dialecte .tw est du Markdown étendu, fidélité
  proche de l'identité) et l'**export DOCX** avec sa limite annoncée
  (notes en texte, LibreOffice requis) ; **l'import DOCX n'est pas livré**
  (les notes perdues au retour = un import mentirait), alternative
  documentée (LibreOffice ou pandoc côté utilisateur).
- Fait : **les deux exports décidés sont livrés** — `src/langage/export.rs`.
  `vers_markdown` = le document au canonique (le test le prouve :
  `parse(markdown) == parse(source)` — pas une copie qui pourrait
  dériver) ; `vers_docx` = HTML puis LibreOffice, **limite verrouillée par
  test** (les notes deviennent `[note: …]`, jamais prétendues notes) ;
  sans LibreOffice, l'état d'erreur dit le geste (« Installez
  LibreOffice, ou exportez en Markdown ») — testé aussi. Branché :
  boutons « Exporter en Markdown… » et « Exporter en DOCX… » (section
  Projets), messages `ExportMarkdown`/`ExportDocx` et leurs choix ;
  épreuves `un_export_markdown_ecrit_le_document_canonique` et
  `un_export_docx_dit_le_geste_ou_ecrit_le_docx` (le DOCX y est réellement
  écrit par LibreOffice).
- Fait : **mesures de stabilité/performance** — rapport livré :
  `agent/Pré-Beta/mesures-performance-2026-11.md` (machine, procédure,
  chiffres). Bancs `bench_full_document` et `bench_bibliotheque_et_stabilite`
  (`src/typst/render.rs`, release). Relevé : parse .tw long (600 blocs)
  1,8 ms, sérialisation 0,2 ms, **stabilité tenue** (le document se relit
  au même), 1 000 notices lues en 1,2 ms, **compilation Typst avec 1 000
  notices 81,7 ms** (contre 31,5 ms sans), raster 3,2 ms/page, pic mémoire
  ~96 Mo. Conclusion : **aucun goulet dans notre code n'est prouvé** — la
  chaîne `.tw` est négligeable, le coût dominant est la compilation Typst,
  déjà contenue par l'architecture (empreintes, pas de recompilation sans
  changement, raster des seules pages visibles, double tampon par hash).
  Les deux leviers futurs sont mesurés et nommés (cache du traitement
  bibliographique Typst, cache incrémental) — rien n'est changé à la
  légère. La ligne « 197 % du budget frame » du banc frère mesure une
  recompilation par frame que l'application ne fait pas.
- **R8 — part automatisable close.** Reste l'acceptation, à l'utilisateur :
  transférer un projet entre les trois systèmes et obtenir un PDF
  comparable ; import interrompu récupérable ; rapport de pertes DOCX
  (déjà livré) ; mesures reproductibles (déjà livrées).

## Retouches du panneau gauche (2026-11-03) — suite

- **Sommaire** : sous la main, le texte se teinte (accent) et le curseur
  pointe (`on_enter`/`on_exit`, `Interaction::Pointer`). Les barres sont
  **dessinées au canevas**, géométrie pure (`segments()`) : un niveau =
  une colonne, tous les verticaux au même x — les lignes « pas alignées
  et difformes » venaient de leur composition dans la marge des widgets.
  Test `les_barres_sont_alignees_et_regulieres`. 495 tests, 115 épreuves.

## La balise `{date:...}` (2026-11-03)

Nouvelle méthode pour les siècles et millénaires, qui **remplace** la
recherche hasardeuse (supprimée) :

- `{date:XIXs}` — un romain (majuscules **ou minuscules**) + `s`
  (siècle) ou `m` (millénaire) ; la règle du **dernier caractère**
  tranche (`IIIm` = millénaire — compter des millénaires en milliers ne
  se fait pas) ; `?` = forme longue (le mot + l'époque) ; `-` en tête =
  av. J.-C. : `{date:-IVs?}` → « IV^e^ siècle av. J.-C. ». Le premier
  dit **`er`** : `{date:Is}` → « I^er^ ».
- `{date:2026.11.03 "dd/MM/yyyy"}` — date pleine + format d'affichage
  **en guillemets**, tokens **anglophones internationaux** (`yyyy yy
  MM M dd d`) ; sans format, celui des normes.
- **Tout se customise dans le panneau des normes** : mot du siècle, mot
  du millénaire, époque après (« ap. J.-C. »/« EC »), époque avant, et le
  format par défaut.
- Parse (`Inline::Date`), génération (petite capitale + exposant par les
  marqueurs survivant à l'échappement), écriture `.tw` aller-retour,
  canonisation. Cœur pur dans `src/langage/dates.rs` (5 tests) +
  parcours complet parse→génération éprouvé.

## La passe des douze demandes sur les normes (2026-11-04)

1. **Le rail** tient un sixième de la largeur (FillPortion 1/5).
2. **Métadonnées = les quick parts de l'auteur** : liste libre, dans
   son ordre (plus de tri alphabétique — `Vec` ordonné + `toml`
   `preserve_order`), aucun champ imposé ; `titre` et `date` sortent de
   l'affichage (tenus ailleurs : le projet, la balise `{date:}`).
3. L'onglet **Page** ne porte plus que la page : « Numéros de page » et
   « Composition du document » supprimés.
4. **Les mesures** refusent le négatif et se lisent en `em` quand
   l'unité manque (`normaliser_mesure`, éprouvée) — saisies et molette.
5. **L'aperçu en direct des titres est retiré** (affichage **et** recal
   Typst à chaque frappe).
6. **La grille des titres** : six niveaux × (taille, gras, italique,
   souligné, casse, espacement) ; la police des titres et la case
   « Titres en gras » sont supprimées ; le gabarit compose italique et
   souligné par niveau.
7. **Figures** : numérotation et style de légende en listes déroulantes
   (« défaut » = vide).
8. **En-tête et pied** : les quick parts du menu s'ouvrent sur les clés
   de métadonnées de l'auteur, dynamiquement.
9. **Couleurs** : cliquer une vignette ouvre le sélecteur pour ce champ
   (`PickerCouleurPourReglage`) ; bordure et remplissage ont leur
   vignette.
10. **Bibliographie** clarifiée : le style, ce que cela donne (aperçu
    juste dessous), puis Zotero — composants communs partout.
11. **Harmonisation** : partout les mêmes briques (section, ligne,
    saisie, choix, case).

## La refonte du panneau des normes (2026-11-04)

Le panneau des normes est repensé de fond en comble, présentation et
confort d'usage :

- **Des onglets verticaux à gauche** (icône + nom, icônes lucide), neuf
  entrées qui se retrouvent d'un coup d'œil : Métadonnées, Page, Texte,
  Titres, Figures et dates, En-tête et pied, Typographie, Couleurs et
  décor, Bibliographie. L'onglet actif est marqué (accent).
- **Le regroupement naturel** : l'ancien onglet « Mise en page »
  — qui entassait tout — est découpé en Page (papier, marges,
  pagination, composition : page de titre et sommaire), Texte (corps,
  interligne, police), Titres (styles et numérotations), Figures et dates
  (légendes, les mots de `{date:...}`). Les couleurs du document
  rejoignent le Décor.
- **Le confort** : la gestion des modèles reste **au-dessus des
  réglages** (NORMS-04-03), resserrée sur une ligne ; le pied porte les
  problèmes, l'accroche et les gestes (Annuler à gauche, Appliquer à
  droite) ; le contenu défile à droite du rail.
- Les entrées mal encastrées (les dates empilées dans une rangée, une
  case dupliquée) sont remises à leur place — le rangement était une
  promesse de la présentation.

## Retouches du panneau gauche (2026-11-03)

- **Hiérarchie** : les entrées (sommaire, chapitres) sont du **texte
  cliquable**, plus des boutons — les gaps venaient du choc des boutons.
  Les traits hiérarchiques sont dessinés (déjà), pas des glyphes.
- **Bibliographie** : création, import, recherche et lien Zotero en
  **icônes** (une rangée), la recherche ne se montre que sur son icône ;
  les gestes note/figure/étiquette/renvoi **retirés** (demande
  explicite).
- **Projets** : recherche en icône, l'étagère en **menu** (« Mon
  étagère », « Tous », les étagères nommées) — pas une bascule. Test
  `une_etagore_nomme_montre_la_sienne`. 494 tests, 115 épreuves.

## Lot bug-report-7 (2026-11-02) — dix-neuvième lot (composeur, fin)

- **4 (jalon 3, fin)** : le composeur d'extension — GUI générique pilotée
  par `parametres()`, insertion au curseur. **bug-report-7.md est
  achevé** : les 23 points traités. 493 tests, 115 épreuves.

## Lot bug-report-7 (2026-11-02) — dix-huitième lot (paquets, jalons 1-2)

- **4 (jalons 1-2)** : paquets Typst résolus localement dans le `World`
  (test d'un `@local` qui compile), trait `ExtensionPaquet` + registre +
  première extension cetz (les paramètres écrivent le code). 492 tests,
  114 épreuves.

## Lot bug-report-7 (2026-11-02) — dix-septième lot (repli, jalon 3)

- **3 (jalon 3, fin)** : la clôture transportable est indentée — elle se
  replie par le mécanisme existant de l'éditeur, sans toucher au vendor.
  Point 3 complet. 490 tests, 114 épreuves.

## Lot bug-report-7 (2026-11-02) — seizième lot (tw unique, jalon 2)

- **3 (jalon 2)** : exporter/importer le projet en un seul `.tw` —
  gestes au menu Projet, reconstruction complète (pièces + accroche),
  refus du futur. Test de va-et-vient. 489 tests, 114 épreuves.

## Lot bug-report-7 (2026-11-02) — quinzième lot (format transportable)

- **3 (jalon 1)** : le format `.tw` transportable — spécification +
  `FichierPortable` (lire/écrire, rétrocompatible, versionné). 4 tests.
  488 tests, 114 épreuves.

## Lot bug-report-7 (2026-11-02) — quatorzième lot (modèles)

- **8** : les modèles n'écrivent plus de Typst caché — document.tw
  visible, normes pour page de titre/sommaire/numérotation, `appliquer`
  pose l'accroche. Règles R5 réécrites sur la règle neuve. 484 tests,
  114 épreuves.

## Lot bug-report-7 (2026-11-02) — treizième lot (métadonnées, fin)

- **9 (fin)** : le dialogue de création propose titre et auteur ; le
  projet naît avec ses métadonnées. Épreuve dédiée. 483 tests,
  114 épreuves.

## Lot bug-report-7 (2026-11-02) — douzième lot (métadonnées)

- **9** : onglet Métadonnées (clé = quick part), sauvegarde au geste,
  et surtout : les entrées `sys.inputs` sont enfin remplies à la
  compilation — le rail existait, personne ne l'alimentait. 483 tests,
  113 épreuves.

## Lot bug-report-7 (2026-11-02) — onzième lot (vignettes)

- **12** : les aperçus de la bibliothèque — le silencieux d'un `main.typ`
  qui échouait ; secours par le document composé, trois cas éprouvés.
  482 tests, 112 épreuves.

## Lot bug-report-7 (2026-11-02) — dixième lot (légendes)

- **23** : libellé, numérotation et style des légendes se règlent dans
  les normes ; la numérotation existait sans jamais être consommée
  (racine du bug). Gabarit + test. 481 tests, 112 épreuves.

## Lot bug-report-7 (2026-11-02) — neuvième lot (barre contextuelle)

- **1** : la barre haute suit le mode — zoom en rendu (créé : −, ajuster
  largeur/hauteur, +), outils d'édition sur le texte seulement, communs
  partout. 480 tests, 112 épreuves.

## Lot bug-report-7 (2026-11-02) — huitième lot (note, latérale)

- **15** : la note s'écrit à son panneau, la seule marque vient au
  curseur. Épreuve + ancienne épreuve réécrite sur la règle neuve.
- **14 (fin)** : les boutons de projet quittent la latérale, comme le dit
  le rapport — le menu de la barre haute les porte. 480 tests,
  112 épreuves.

## Lot bug-report-7 (2026-11-02) — septième lot (projets par étagère)

- **18** : la liste des projets montre l'étagère du projet courant,
  bascule « Tous », barre de recherche par nom. Filtre pur éprouvé
  (2 tests). 480 tests, 111 épreuves.

## Lot bug-report-7 (2026-11-02) — sixième lot (barres hiérarchiques)

- **13** : les connecteurs du sommaire passent des glyphes de boîte à de
  vrais traits — la diagonale venait de leur dérive dans une police
  proportionnelle. Un niveau = une colonne fixe. 478 tests, 111 épreuves.

## Lot bug-report-7 (2026-11-02) — cinquième lot (menu Projet)

- **14** : menu déroulant « Projet » dans la barre supérieure
  (`iced_aw::menu`, ajout de la dépendance — `cargo deny check` vert) :
  modèles, archive, sauvegarde, exports. Boutons latéraux gardés.
  478 tests, 111 épreuves.

## Lot bug-report-7 (2026-11-02) — quatrième lot (normes)

- **20/21** : la fenêtre des normes passe en deux colonnes de portions —
  contrôle aligné au même x, qui se resserre avec la fenêtre ; champs 260
  en `Fill`.
- **19** : la fenêtre des normes entre dans le contrat des fenêtres
  (Tabulation circule, Maj+Tabulation revient) — épreuve de câblage.
  478 tests, 111 épreuves.

## Lot bug-report-7 (2026-11-02) — troisième lot (roue du thème)

- **11** : la roue sert aussi aux champs de l'éditeur de thème (cible
  `PickerCouleur.champ`) ; l'ancien picker à curseurs est retiré —
  remplacement demandé, tracé. Épreuve dédiée. 478 tests, 110 épreuves.

## Lot bug-report-7 (2026-11-02) — second lot (roue)

- **10/17 (color wheel)** : curseur en croix, champ hex copiable/collable
  qui pilote la roue, RGB en direct, palette de 8 couleurs rapides qui
  appliquent sans fermer. Épreuve dédiée. 478 tests, 109 épreuves.

## Lot bug-report-7 (2026-11-02) — premier lot

État des 24 points : `agent/bug-report-7-etat.md`. Faits et vérifiés :

- **5 (siècles)** : `#smallcaps[XX]` était écrit avant l'échappement Typst
  — le code mort apparaissait dans le rendu. Règle neuve : capitale +
  marqueur **en exposant**, écrits après l'échappement (marqueurs de
  contrôle), déclencheur resserré, backtique = déclaration explicite.
  4 tests.
- **7 (étiquette)** : le refus `.tw` était périmé (`<nom>` est lu depuis
  R5) — le geste écrit `<>` dans les deux dialectes, curseur dedans.
  Épreuve + ancienne épreuve réécrite sur la règle neuve (changement
  voulu, pas une régression).
- **16 (boutons de format morts)** : sur un bloc **vide** (projets neufs,
  modèles) les boutons ne faisaient rien en silence. Un bloc vide se
  forme maintenant. 4 épreuves.
- **18 (document d'écriture)** : le bouton répond (« déjà sous les yeux »)
  et ramène depuis un chapitre. Épreuve.
- **22 (bouton accrocher)** : intitulé explicite.
- **6 (refresh au basculement)** : `TogglePreview` recompile à la
  réapparition, sans purge.

## Lot couleur/quick parts (2026-11-02)

- **Color picker refait** : une roue de teinte + un carré
  saturation/valeur (canevas interactifs), centrée dans la fenêtre — plus
  la liste de noms rejetée en bas à gauche. Conversions HSV <-> RVB <-> hex
  pures et testées ; ouverture sur la dernière couleur ; la fermeture
  n'écrit rien. `src/ui/picker_couleur.rs`, 3 tests.
- **Quick parts d'en-tête/pied** : trou réparé — `{document}`, `{date}`,
  `{auteur}`, `{projet}` rendaient vide (`sys.inputs` jamais alimenté ;
  `compile_avec_entrees` + `Library::builder().with_inputs`) ; `{titre}`
  ajouté (résolu par le moteur, `query` sur les titres) ; images dans
  l'en-tête/pied (`![...](...)`, hauteur de logo). Les marqueurs littéraux
  `\x` étaient défigurés par l'échappement (leurs substitutions ne
  matchaient jamais) — passés en vrais caractères de contrôle. Interface :
  un sélecteur de quick parts par champ dans les normes. Épreuve au PDF
  extrait : `{projet}` et `{titre}` vivent dans l'en-tête, l'image s'y
  pose sans erreur.
- **Liste des fonctions Typst restantes** :
  `agent/Pré-Beta/typst-restant-a-offrir-2026-11.md` — les deux plus
  faciles choisis et faits (quick parts, couleur libre).
- **Non fait, dit pour ce qu'il est** : le GUI cetz exige d'abord que le
  moteur résolve les paquets Typst (audit R5) — le chantier est décrit en
  fin de liste. Un lot entier à part.

## Balayage final (2026-11-02) — ce qui avait été laissé de côté

- **R8 « import interrompu récupérable » est désormais prouvé** : l'import
  se passe dans un atelier voisin puis rejoint la destination par
  renommages ; toute panne remet la destination **exactement** comme avant
  (rien de neuf ne reste, ce qui était remplacé revient de la corbeille).
  Épreuve rouge puis verte `un_import_interrompu_reste_recuperable` (un
  fichier impossible à créer casse l'import au milieu : état d'avant
  retrouvé octet pour octet).
- **R6 « gestes souris » est éprouvé** : `un_glissement_de_separateur_regle_la_disposition`
  — l'origine du glissement est la position réelle du clic, les deltas
  partent de l'origine (jamais cumulés), les bornes tiennent, le
  relâchement rend la main.
- **R9 « centre d'aide/diagnostic et récupération intégrés » est finalisé**
  dans l'aide (`F1`) : section **Diagnostic** (`diagnostic_resume`, une
  seule source de vérité — version, serveur de langage avec son remède,
  diagnostics, bibliographie) et section **Récupération** (les temporaires
  perdus reviennent seuls à l'ouverture ; boutons « Ouvrir la corbeille »
  et « Ouvrir les sauvegardes du projet »). Épreuve
  `l_aide_dit_le_diagnostic_et_la_recuperation`.
- **Le cours « produire et récupérer un devoir »** (exigence R9) est
  éprouvé d'un tenant : `un_devoir_se_produit_et_se_recupere` — le devoir
  s'écrit, une autre main touche le fichier, le conflit confirmé garde ce
  qui allait être perdu, et la version perdue se relit dans les
  sauvegardes. Faux départ documenté : les copies de versions sont
  **espacées d'une minute** (design voulu, `force` passe) — une épreuve qui
  attend deux copies en une seconde teste ce que le design ne promet pas.
- Ce qui reste hors de cette plateforme, inchangé : les acceptations
  visuelles (R6 contraste/petites fenêtres), les acceptations d'usage
  (transferts, parcours d'installation réels), la CI et les essais natifs
  Windows/macOS.

## R2 (terminé) — import sans perte et source fiable

- Fait : l'import Typst travaille sur le tampon ouvert, avec aperçu de la conversion (convertis / restés bruts / avertissements) avant d'entrer dans le texte ; application d'un seul geste, annulable d'un seul ; le fichier d'origine n'est jamais touché ; aucune écriture avant la sauvegarde. Épreuves rouges puis vertes : frappes non sauvegardées préservées, annulation sans effet de bord, annulation d'un seul geste (undo/redo).
- Fait : un fichier `.tw` rejoint lui aussi le tampon ouvert après aperçu, et un tampon modifié n'est jamais remplacé en silence par la copie d'ouverture ; rouvrir son propre document avec des changements non sauvegardés le dit au lieu de l'écraser.
- Fait : les tableaux irréguliers ne tronquent plus aucune cellule — la plus large des lignes élargit le tableau (en-tête et alignements suivent), comblement des trous inchangé ; round-trip stable, unicode intact. La troncature (`resize` en `parse.rs`) était un effacement silencieux de données ; épreuves rouges puis vertes.
- Fait : la suppression de notice est une **vraie** corbeille — l'entière part à la corbeille (`notice-<clé>.bib`) avant toute suppression, et si la copie échoue la bibliographie ne bouge pas. La notice supprimée se restaure d'un geste (bouton du panneau bibliographie, `ReferenceRestaurer`) ; une clé déjà présente ne se dédouble jamais. Avant, la notice était détruite pendant que le message promettait la corbeille. Épreuves : corbeille réelle, restauration.
- Fait : le renommage LSP dit son périmètre (l'aperçu nomme déjà chaque fichier touché) et son **résultat détaillé** : les fichiers touchés sont nommés dans le rapport, pas seulement comptés. Et un fichier illisible arrête le renommage au lieu d'être glissé en silence — un renommage « appliqué » qui aurait sauté un fichier était un demi-renommage invisible. Épreuves rouges puis vertes.
- Fait : les images homonymes ne s'écrasent jamais — la copie prend un nom libre (`photo-2.png`), l'existante reste intacte et le document cite la copie ; épreuve d'acceptation posée (`une_image_homonyme_ne_crase_pas_la_premiere`, comportement existant désormais garanti).
- Fait : un import dont le **fichier d'origine a changé** depuis l'aperçu se refuse — l'empreinte du fichier est prise à l'aperçu et revérifiée à la décision ; jamais de texte périmé dans le tampon. Épreuve rouge puis verte (`un_import_annule_si_le_fichier_a_change_depuis_l_apercu`).
- À faire (R2) : aperçu d'import en panneau plutôt qu'en question, erreurs à mi-opération (le conflit interne tampon est déjà refusé).

## R3 (terminé) — références locales correctes

- Fait : une citation inconnue ne se cache plus. Les deux voies de neutralisation (`@cle` et `#cite(<cle>)`, dans le texte, les chapitres inclus et le texte généré depuis le `.tw`) produisent désormais un **diagnostic** — clé, ligne, fichier — qui rejoint les avertissements de compilation : le panneau de l'aperçu et la barre d'état les montrent, au lieu du silence d'un `[cle]` nu. Épreuves : citation inconnue signalée et jamais transformée en fausse référence, `#cite` inconnu signalé, une citation perdue dans un chapitre nomme **son** fichier. Les fonctions sans diagnostic sont cantonnées aux tests.
- Fait : les clés non ASCII sont des clés à part entière (même alphabet que le parseur) — et une clé non ASCII **manquante** se neutralise au lieu de casser la compilation. Une adresse électronique n'est **jamais** une clé : le `@` collé à un mot et suivi d'un point s'échappe (`\@`), Typst la lit comme du texte — avant, `user@example.com` devenait `user[example].com` alors que le commentaire du code promettait le contraire. Le point final d'une citation reste de la ponctuation (`@vivante.` n'est pas la clé `vivante.`). Le composeur ne s'ouvre plus sur une adresse. Épreuves : `une_cle_non_ascii_manquante_ne_casse_jamais_le_document`, `une_cle_non_ascii_vivante_reste_une_vraie_reference`, `une_adresse_electronique_nest_jamais_une_cle`. Limite connue : un `@` dans l'URL d'un lien (`https://…/@user`) reste neutralisé comme aujourd'hui — à traiter avec les liens.
- Fait : « dernier aperçu valide clairement indiqué comme ancien » — un échec de compilation conserve le dernier bon rendu (promesse du double tampon, tenue) mais désormais, l'aperçu affiché sous des erreurs porte la mention « Aperçu ancien — la compilation actuelle a échoué ». `RenderState::apercu_perime()` en est la vérité testée ; épreuve avec une vraie compilation du projet témoin (`un_apercu_affiche_sous_des_erreurs_est_indique_ancien`).
- Fait : la recherche du composeur de citation trouve l'**auteur, le titre, l'année et la clé** — elle ne cherchait que la clé, ce qui rendait injouable la moitié des notices (on cite un ouvrage par son auteur, rarement par sa clé). Le texte de recherche est construit depuis la même `BibliographyEntry` que la bibliothèque visible (source partagée) ; épreuve `le_filtre_cherche_auteur_titre_et_annee`. La recherche du panneau bibliographie couvrait déjà ces champs.
- Fait : les épreuves d'acceptation R3. **Deux bibliographies** : les clés d'une bibliographie portée par un chapitre étaient neutralisées à tort (`cles_vivantes` ne balayait que l'assemblage et le disque — pas les `#include` en mémoire) ; elle balaye désormais l'arbre des inclus, les directives des chapitres se résolvant à leur place. Et une **clé commune à deux bibliographies** produit un diagnostic de collision nommant la clé et les deux fichiers — « détection des erreurs et collisions » — au lieu d'une ambiguïté muette. Épreuve `deux_bibliographies_a_cles_communes_ne_se_contredisent_pas` (rouge puis verte) : rien n'est neutralisé, la collision se dit, la clé sans ambiguïté se tait, et deux bibliographies complémentaires compilent sans erreur. **Texte extrait du PDF** : épreuve `le_texte_extrait_du_pdf_porte_la_source_citee` — le PDF produit par le vrai moteur est passé par `pdftotext`, et le texte extrait porte l'auteur et le titre de la source citée (notes + bibliographie) ; la comparaison ignore la casse (la bibliographie met les titres en capitales).
- Fait : **auteurs multiples** lus dans le texte extrait du PDF réel (`les_auteurs_multiples_se_lisent_dans_le_texte_extrait`), et **compilation en erreur** : les erreurs se disent avec le message du document et les avertissements de préparation restent visibles à côté, rien n'est avalé (`une_compilation_en_erreur_dit_erreurs_et_avertissements`).
- Fait : le **contrat de source bibliographique partagé est écrit** — `agent/Pré-Beta/contrat-source-bibliographique.md` : la source unique (`BibliographyEntry` des `.bib` cités, y compris ceux des chapitres), ses consommateurs (bibliothèque, recherche, complétion/composeur, citations, aperçu, PDF), sept invariants nommant chacun son épreuve, et ce qu'il ne couvre pas encore (import `.bib`, `@` dans une URL, Zotero en écriture).
- À faire (R3) : import d'un `.bib` local (choix/fusion) — le dernier livrable R3 non couvert.

## R4 (terminé) — styles CSL et Zotero fiables

- Fait : **le style du projet, ses règles de priorité**. Le style des citations et de la bibliographie vit dans les normes (`mise-en-page.toml`, `[bibliographie] style = …`), choisi par le nouvel onglet « Bibliographie » de la fenêtre des normes — parmi les styles que le moteur embarqué connaît réellement (vitrine validée par `ArchivedStyle::by_name`, jamais une liste divergente), avec son type annoncé (notes, auteur-date, numéroté). Règles explicites : une directive sans style reçoit celui du projet ; une directive qui impose **son** style reste maîtresse et le conflit se dit avec les deux noms et qui fait autorité ; un style refusé par le moteur n'est jamais appliqué (diagnostic + style de base). Changer de style ne touche jamais aux clés ni aux notices. Épreuves : `le_style_du_projet_s_applique_a_la_bibliographie`, `une_directive_qui_impose_son_style_ne_se_ecrase_jamais`, `un_style_de_projet_inconnu_est_dit_pas_applique`, `changer_le_style_ne_touche_jamais_aux_notices`, `les_styles_proposes_sont_ceux_du_moteur`.
- Fait : **l'import d'un `.csl` local**. Contrôlé par le **même analyseur que le moteur** (`citationberg`, comme `IndependentStyle::from_xml`) : indépendant → copié sous `styles/` du projet avec un nom libre (jamais d'écrasement), l'original ne bouge pas, et le choix devient `styles/…csl` lu dans la directive ; **dépendant** → résolution par son parent (voir ci-dessous) ; invalide, trop grand (2 Mo) ou non UTF-8 → dit, rien ne change. Le bouton « Importer un fichier .csl… » vit dans l'onglet Bibliographie. Épreuves : `le_controle_csl_dit_independant_dependant_et_invalide`, `un_nom_avec_extension_est_un_fichier`, `un_style_csl_importe_se_lit_dans_la_directive`, `un_style_dependant_sans_parent_dit_le_geste`.
- Fait : **première course d'acceptation R4** — `les_trois_familles_rendent_des_textes_differents` : la même notice citée rend trois textes de PDF distincts sous auteur-date (apa), numérique (ieee) et notes (chicago-notes), comparés sur le PDF réel extrait (`pdftotext`), pas seulement au compilateur.
- Fait : **l'aperçu citation + bibliographie**. La fenêtre des normes montre un petit jeu représentatif — une notice, une citation avec localisateur, une seconde référence, et la bibliographie — **rendu par le moteur même** (texte extrait des cadres compilés, jamais une imitation ; l'étude interdit un aperçu qui ne serait pas le style final). La ligne dit explicitement quel style est regardé ; un style refusé par le moteur n'a pas d'aperçu qui ment (« Le moteur refuse ce style »). L'aperçu se refait à chaque changement de style, y compris pour un `.csl` du projet. Épreuve `l_apercu_est_rendu_par_le_moteur` : la source citée s'y lit, les trois familles se distinguent, un style inconnu n'a pas d'aperçu.
- Fait : **les styles dépendants se résolvent**. Le parent est cherché où il peut vivre — les `styles/` du projet d'abord, les styles installés dans Zotero ensuite (par identifiant CSL, jamais en devinant) ; trouvé, il est copié dans le projet (Zotero n'est jamais modifié) et le choix devient le parent, **l'écart de locale étant dit** : le moteur n'exécute que des styles indépendants, c'est la locale du document qui sert. Trouvé nulle part : le style dépendant est conservé sous son nom (« conserver le nom demandé »), **rien n'est choisi** (le dernier rendu valide reste), et une question propose explicitement le téléchargement du parent depuis le dépôt des styles — jamais sans consentement, jamais en silence ; refuser laisse tout en l'état avec le geste expliqué (importer le parent à la main). Le téléchargement (`ureq`, HTTPS) se dit, un échec n'empêche rien. Épreuves : `un_style_dependant_sans_parent_dit_le_geste`, `un_parent_du_projet_resout_le_style_dependant` (avec l'écart de locale), `un_parent_de_zotero_resout_le_style_dependant` (copie dans le projet, Zotero intact), `le_telechargement_lit_le_style` (serveur local, sans réseau).
- Fait : **les styles installés dans Zotero sont la troisième porte du choix**. La fenêtre des normes les liste (titre + identifiant) depuis le dossier de styles de Zotero ; un choix les traite exactement comme un import — **copie dans le projet** (Zotero n'est jamais modifié, éprouvé), validation par le moteur, résolution du parent s'il est dépendant (depuis la fratrie du fichier Zotero, où le parent vit en général), et le choix devient un chemin du projet. Absents : la liste le dit (« aucun style installé, ou Zotero absent ») — jamais de silence. **Style manquant au moment du choix** (désinstallé, fichier disparu) : diagnostic, rien n'est choisi, aucun remplacement discret. Les trois issues de tout import de style partagent désormais une seule aide (`appliquer_style`) : choisi, parent manquant (la question de téléchargement avec consentement), refusé (rien ne change). Les écrits de style rejoignent le contrôle « modifie le projet » (comme l'image). Épreuves : `un_style_zotero_choisi_se_copie_et_se_choisit`, `un_style_zotero_disparu_est_un_diagnostic`.
- Fait : **Zotero en lecture seule — transaction et rattachement au projet**. L'export produisait trois fichiers l'un après l'autre (clés, bibliothèque, liaison) : une panne laissait un projet mi-écrit — clés revendiquées sans bibliothèque, liaison sans clés. Désormais `ecrire_en_transaction` (`src/project/mod.rs`) pose tout en temporaires, écarte les anciennes versions, met les neuves en place, et **remet exactement l'état d'avant** au moindre échec ; une cible qui n'est pas un fichier est un refus immédiat (jamais un dossier écarté — trou trouvé par l'épreuve rouge). L'export est aussi **rattaché au projet demandeur** : le message porte le projet de la requête, et une réponse arrivée après un changement de projet n'écrit rien — elle le dit. Épreuves : `une_transaction_ecrit_tous_les_fichiers_ensemble`, `une_transaction_faillie_ne_change_rien`, `un_export_zotero_en_cours_ne_s_ecrit_pas_dans_un_autre_projet`, `un_export_zotero_ecrit_ses_fichiers_en_transaction`. Déjà en place et vérifiés : la pagination (100 par 100, collections et notices), la fidélité des métadonnées (BibTeX composé par nous, titres nus) et les clés stables (`zotero/keys.rs`).
- Fait : **l'import BibLaTeX sans plugin** (l'alternative de R4) : Zotero sait exporter lui-même un `.bib` — le bouton « Importer un fichier .bib… » du panneau bibliographie le rejoint à la bibliothèque locale, sans plugin ni API. Les clés communes ne s'écrasent **jamais** : une notice identique est ignorée (« déjà là »), une notice différente est conservée et **dite** (conflit, la première tient) ; le rapport nomme les trois cas. La bibliothèque Zotero (`references-zotero.bib`) n'est jamais visée — elle, se lit seulement. Un fichier sans notice lue est refusé, rien n'entre. **Trou de fidélité trouvé et réparé au passage** : le parseur BibTeX traitait un `@` dans un commentaire `%` comme une notice — l'exemple commenté du modèle devenait un fantôme dans la bibliographie (compté, affiché, citable) ; désormais un `@` commenté n'ouvre jamais de notice (épreuve rouge puis verte `un_exemple_commente_nest_jamais_une_notice`). Épreuves : `un_import_bib_rejoint_la_bibliographie`, `un_import_bib_ne_ecrase_jamais_une_notice`, `un_import_bib_sans_notice_dit_le_probleme`, `un_import_bib_sans_plugin_rejoint_la_bibliographie` (parcours complet, rapport lu).
- Fait : **les courses d'acceptation R4**. Redémarrage hors ligne : le style du projet survit au redémarrage et se rend sans réseau (`un_projet_relu_hors_ligne_conserve_son_style`). Coupure Zotero en cours de requête : rien n'est écrit, l'erreur se dit et n'annonce pas un réglage (`une_coupure_zotero_n_ecrit_rien_et_le_dit`). Échec de stockage : la transaction renonce en entier, « rien n'a changé » (`un_echec_de_stockage_ne_laisse_rien`). Moteur verrouillé : les cinq briques Typst sont **épinglées exactement** (`=0.15.1` dans `Cargo.toml` — elles ne l'étaient pas, un glissement mineur aurait changé les rendus) et l'épreuve `le_moteur_typst_est_verrouille` casse au moindre déverrouillage. Les autres points d'acceptation étaient déjà couverts : trois familles de style comparées au PDF (avec localisateurs et références multiples), style intégré/indépendant/installé/dépendant, parent absent, fichier invalide, changement de projet pendant une requête, styles Zotero, écarts CSL connus signalés.
- **R4 clos.** 440 tests ordinaires, 79 épreuves d'espace, portes strictes vertes (fmt, clippy -D warnings, build, cargo deny, diff --check).

## R5 (terminé) — devoir et thèse prêts à remettre

- Fait : **les modèles de documents, en un geste** (premier livrable). Le panneau des projets propose « Nouveau devoir court… » et « Nouveau travail long… » — un clic, et le projet naît complet : page de titre (métadonnées de l'écrivain, **aucune institution imposée**), sommaire, sections, bibliographie. Le travail long ajoute les préliminaires à numérotation romaine, le corps numéroté et **l'en-tête courant résolu par le moteur lui-même** (`header: context`) — jamais un champ imité. Les deux gabarits sont écrits pour être relus et modifiés à la main. Le modèle `Classique` (l'assemblage d'origine) reste la voie de `NewProject` et de la migration : **un projet existant garde exactement son rendu** sans choix explicite (éprouvé). Épreuves : `les_deux_modeles_compilent`, `le_devoir_promet_page_de_titre_sommaire_et_bibliographie` (promesses lues dans le **PDF extrait**), `le_travail_long_resout_son_entete_courant` (le titre apparaît page de titre **et** en-tête), `un_geste_cree_le_projet_du_modele_choisi`, `un_projet_existant_garde_son_rendu`. Piège trouvé et documenté : `compile`/`render_pdf` du moteur prennent le **texte source** en entrée, pas un nom de fichier — un test qui passait « main.typ » compilait littéralement cette chaîne (faux vert) ; corrigé. Piège des tests : deux épreuves partageaient un dossier nommé par le pid, l'une effaçait celui de l'autre — chaque appelant a désormais le sien.
- Fait : **typographie et titres — validation cohérente par niveau, et application réelle prouvée**. La validation existante (`problemes()`, déjà montrée par la fenêtre) ignorait les réglages **par niveau** : `casse_2` fausse, `couleur_3` invalide ou `espacement_1` illisible partaient en silence quand le même défaut au global était dit — l'édition par niveau n'était pas cohérente avec sa validation. Désormais les six niveaux sont validés comme les globaux (casse, espacement, couleur), chacun nommé (« casse du titre 2 »…). **Application réelle prouvée au PDF extrait** : `la_casse_par_niveau_s_applique_au_rendu` (le niveau 1 en capitales dans le PDF, le niveau 2 inchangé — comparaison en casse réelle, pas en minuscules) et `les_champs_dynamiques_se_resolvent_par_le_moteur` (« p. {page} — {pages} pages » devient « p. 1 — 2 pages », « p. 2 — 2 pages » dans le PDF : les champs dynamiques des en-têtes/pieds sont bien résolus **par le moteur**, jamais imités — l'audit de R5 est ainsi éprouvé pour `{page}`/`{pages}`). Épreuve `les_reglages_par_niveau_sont_valides_comme_les_globaux`.
- Fait : **l'aperçu des titres, rendu par le moteur**. L'onglet Titres de la fenêtre des normes montre les six niveaux **réellement rendus** avec les normes du dialogue (`apercu_titres`, `src/mise_en_page/mod.rs`) — un échantillon compilé par le moteur du projet et rendu en image, le gabarit passant par `overrides` : le dialogue montre ce qui sera **sans rien écrire au projet** avant l'application. L'aperçu se refait à chaque changement (saisie, molette, bascule). Indisponible (normes refusées), il le dit — jamais d'aperçu muet. Épreuve `l_apercu_des_titres_est_rendu_par_le_moteur` : mêmes normes → mêmes pixels (le moteur est déterministe), une norme différente → image différente — les pixels **portent** les normes, preuve que ce n'est pas une imitation.
- Fait : **notes, légendes et références croisées — vérifiées au rendu, et un trou réparé**. Le correcteur neutralisait les **vrais renvois** : `@fig:une` était lu comme la clé « fig » puis déguisé en `[fig]` — le renvoi mourait et la compilation n'en disait rien. Racine corrigée dans `corriger_le_typst` : un `@` à clé à deux points (`@fig:une`) est un **renvoi** (jamais une citation), et une étiquette définie dans le document (`<chapitre>`, collectée dans tout l'arbre des inclus par `cles_vivantes`) l'est tout autant — les deux traversent le correcteur intacts. Une **cible manquante** n'est plus neutralisée : elle passe au moteur, qui la **diagnostique** par son nom (R5, « diagnostics ») au lieu d'un faux `[cle]` silencieux. Vérifié au PDF extrait : `une_note_de_bas_de_page_se_lit_dans_le_pdf`, `une_legende_de_figure_se_lit_dans_le_pdf` ; au pipeline : `un_renvoi_a_une_figure_se_resout_au_rendu`, `une_etiquette_du_document_est_un_renvoi`, `un_renvoi_sans_cible_est_diagnostique`.
- Fait : **l'audit de `cetz`, exigé avant toute promesse de graphiques** — éprouvé par `cetz_ne_se_charge_pas_dans_le_moteur_embarque` : le monde de rendu ne résout **pas** les paquets Typst (`@preview/…` cherche un fichier du projet, « file not found »). **Les graphiques cetz ne sont donc pas fiables aujourd'hui** : ils ne sont pas présentés comme tels, et quiconque importe cetz reçoit le diagnostic du moteur (fichier/ligne), pas un silence. La garde casse le jour où les paquets seront résolus — signal de réévaluation honnête, pas une promesse en avance.
- Fait : **les gestes d'insertion** (R5, « insertion »). Le panneau bibliographie propose « Note », « Figure », « Étiquette », « Renvoi » : le gabarit entre **au curseur**, et le curseur se pose **là où l'on doit écrire** — entre les crochets de la note, sur la légende de la figure, dans les chevrons de l'étiquette ; le renvoi pose `@`, qui ouvre la complétion des étiquettes. L'aide `insert_avec_curseur` (`src/editor/mod.rs`) compte le recul dans la même unité que le curseur (octets), sans conversion faillible. Épreuve `les_gestes_d_insertion_placent_texte_et_curseur` : les quatre gestes, texte **et** position du curseur vérifiés. Première épreuve trompeuse : les gestes s'empilaient au curseur — comportement **voulu** (chaque insertion part du curseur), c'est l'épreuve qui supposait le bout du tampon ; corrigée en posant le curseur entre les gestes.
- Fait : **les gestes d'insertion parlent le dialecte du tampon** (trou trouvé par la course) : le document `.tw` compose en **échappant** le balisage Typst (c'est sa nature : le texte reste le vrai), donc `#footnote[]` y devenait du texte mort. Les gestes sont devenus dialectaux — note `^[…]` sur `.tw`, `#footnote[]` sur les chapitres `.typ` ; figure : le sélecteur d'image sur `.tw` (la figure légendée du format), le gabarit `#figure` sur `.typ` ; étiquette : ` <>` sur `.typ`, et sur `.tw` **le geste s'explique** (« les étiquettes s'écrivent dans les chapitres .typ ») au lieu d'insérer du mort — la syntaxe d'étiquette du `.tw` reste à inventer, dite comme telle. Épreuve `les_gestes_d_insertion_placent_texte_et_curseur` réécrite sur les deux tampons.
- Fait : **les courses d'acceptation R5**. `le_devoir_synthetique_tient_ses_promesses` : le modèle « devoir court » créé d'un geste, une notice, un chapitre (Typst brut) avec note exacte, figure légendée, renvoi, citation — et dans le PDF extrait : titres, sommaire, sauts (≥3 pages), note, légende, bibliographie, renvoi résolu (jamais neutralisé), l'export = le PDF du moteur. `la_these_synthetique_tient_ses_promesses` : le modèle « travail long », chapitres avec renvois de figure, de **tableau** et de **titre**, note, en-tête courant vérifié (le titre page de titre **et** en-tête), sauts (≥4 pages) — trouvé et corrigé au passage : Typst refuse de référencer un titre sans numérotation → le modèle de travail long numéroté ses chapitres (`set heading(numbering: "1.1")`), comme toute thèse. La réouverture/migration sans changement de rendu était déjà éprouvée.
- **R5 clos.** 453 tests ordinaires, 84 épreuves d'espace, portes strictes vertes (fmt, clippy -D warnings, build, cargo deny, diff --check).
- Limite dite : la syntaxe d'étiquette/renvoi du document `.tw` n'existe pas encore (le geste l'explique) ; les renvois vivent dans les chapitres `.typ`, où ils sont éprouvés. Les cours de vérification humaine restent à l'auteur (rendus visuels).

## R6 (en cours) — édition quotidienne et qualité d'interface

- Fait : **la recherche et le remplacement « littéral et regex exacts »** (trou réel). Le comptage du mode littéral échappait bien le motif, mais **le remplacement relisait le motif brut en regex** : chercher `a.b` en littéral trouvait bien `a.b`, mais le remplacer traitait `.` comme « n'importe quel caractère » — et `$1` s'y développait comme un groupe même en littéral. Le mode est maintenant **dans l'API du moteur** (`src/app/recherche.rs` : `occurrences`, `remplacement`, `remplacer_tout`, `motif_effectif`) : en littéral, ni les métacaractères du motif ni les `$` du remplacement ne signifient rien de plus que ce qu'ils sont ; en régulier, les groupes se rappellent par `$1` et le motif invalide se dit. Épreuves : `le_litteral_ne_lit_pas_les_metacaracteres` (le point et la parenthèse littéraux), `le_remplacement_litteral_garde_ses_dollars` (`$1` reste `$1`), les quatre épreuves régulières conservées. Noté au passage : les occurrences ne se chevauchent pas (règle de la recherche régulière, comme partout).
- Fait : **l'aperçu des changements et l'undo** (R6). La barre de recherche dit ce que l'occurrence courante devient — « « chat » devient « chien » » — les groupes déjà rappelés dans le remplacement, recalculés à chaque frappe ; un motif cassé n'a pas d'aperçu qui mente (`apercu`, pur, dans `src/app/recherche.rs`). L'undo est éprouvé sur les deux gestes de l'acceptation : `un_remplacement_s_annule_d_un_geste` (tout remplacer est **un** geste annulable, et l'aperçu avait dit le changement avant qu'il n'arrive) et `une_note_supprimee_se_retrouve_a_l_annulation` (la note part en une frappe sur sa sélection, l'annulation la remet entière).
- Fait : **raccourcis sans conflit avec la saisie, et annulations sans enregistrement prématuré**. Le plan de touches (`plan_touche`) n'était pas couvert : il est maintenant éprouvé par `tests_plan_touche` — toute frappe **sans** Contrôleur ne lance aucun raccourci (les 17 touches du plan, avec et sans Maj : la saisie reste à la saisie), les raccourcis annoncés font leur geste (Ctrl+S, Ctrl+F, Ctrl+1, F1 = aide), et Contrôleur+Alt laisse les touches au système. Le garde existait (Ctrl exigé) — il est désormais verrouillé. **Les aperçus n'enregistrent rien prématurément** : `les_reglages_annules_ne_s_enregistrent_pas` (normes changées dans le dialogue, fermées sans appliquer → `mise-en-page.toml` octet pour octet intact, et la réouverture montre l'ancienne valeur) et `le_theme_annule_ne_s_enregistre_pas` (couleur changée dans la copie de travail de l'éditeur de thème, fermé sans enregistrer → le fichier du thème intact). Les deux dialogues suivaient déjà « le fichier ne bouge qu'à l'enregistrement » — c'est désormais une garantie éprouvée, pas une intention.
- Fait : **onglets, listes longues et mode focus éprouvés**. `un_changement_d_onglet_alterne_les_vues_sans_perdre_le_tampon` : les deux onglets alternent dans les deux sens et **aucun changement ne perd le tampon** — et le modèle des vues est désormais clair dans l'épreuve : deux vues d'un même document **partagent** son tampon (le contrat « documents, vues, session cohérents »), la vue active vit dans `state.editor`, les vues garées dans `espace.vues`. `une_longue_liste_bibliographique_restee_vivante` : 300 notices entrent sans troncature et le filtre tient. `le_mode_focus_rend_la_disposition_d_avant` : entrer et sortir du mode focus rend la disposition **exactement** comme avant, le contrat écrit de `LayoutState` devenu une garantie éprouvée. Premières versions trompeuses documentées : `NewFile` rouvre toujours le même `document.tw` (pas une deuxième vue), et `open_document(.., nouvel_onglet)` ne se prête pas à l'écriture de tampon dans les épreuves — la primitive éprouvée est `VueDupliquer`.
- Fait : **sélection de bibliographie et de notes éprouvées**. `la_selection_de_bibliographie_ouvre_la_bonne_notice` : choisir une notice dans la liste ouvre **sa** fiche (la bonne clé, le bon type), et fermer sans rien écrire ne touche aucune notice. `la_selection_de_note_s_ecrit_a_la_fin` : le composeur se saisit d'un auteur, le choix s'écrit **dans le texte** (`@dupont2023`) et le composeur se ferme en écrivant — la promesse du composeur (« la note se choisit, se pagine, et s'écrit à la fin ») devenue garantie. Ce qui manquait était découvert en cours de route : ni `ModifierReference` ni le composeur n'avaient AUCUNE épreuve d'intégration.
- Fait : **diagnostics sans conflit avec la saisie** — `un_saut_de_diagnostic_ne_touche_pas_la_saisie` : le saut à un diagnostic (`GotoError`) déplace la vue sans toucher au texte, la saisie continue d'aller au tampon, et **l'annulation ne révèle aucun geste fantôme** (le saut n'en ajoute aucun). Avec `plan_touche` (tout raccourci exige le Contrôleur) et l'aide F1 couverts, le second volet de R6 (« raccourcis, diagnostics, menus et aide, sans conflit avec la saisie ») est tenu ; les menus de la barre d'outils sont des listes de choix qui n'agissent qu'au clic, sans surface clavier.
- **R6 — part automatisable close.** Reste la part visuelle, réservée à l'utilisateur : contraste, petites fenêtres, et l'acceptation (parcours réel sur trois tailles de fenêtre et thèmes clair/sombre, clavier/souris ; recherche avec métacaractères et `$`, changement d'onglets, suppression/undo de notes, sélection de thème, liste bibliographique longue — chaque point a désormais son épreuve, le parcours réel reste à faire).

## R7 (premier incrément livré) — abstractions portables

- Fait : **le dossier des données et l'ouverture système sont portables**.
  `dossier_donnees()` (`src/project/mod.rs`) : sous Unix — Linux comme
  macOS — le chemin historique `~/.typst-writer` **reste** (les projets
  existants n'ont pas déménagé, c'était la contrainte de sûreté) ; sous
  Windows, `%APPDATA%\typst-writer`, avec `USERPROFILE` en repli.
  `commande_ouverture()` + `ouvrir_chemin()` : `xdg-open`/`open`/`explorer`
  selon la plateforme — le seul appel en dur (`xdg-open` dans
  `src/app/themes.rs`) est remplacé. Tests `epreuves_portabilite` :
  le dossier suit la plateforme, la commande est celle de la plateforme
  (testée, pas devinée). Doc : chemin Windows ajouté à
  `docs/guide/fichiers.md`.
- Note : les épreuves d'espace tournent **déjà** sans Zotero ni tinymist
  (HOME jetable, aucun démon, aucun binaire LSP) — « utile sans Zotero ni
  serveur de langage » est couvert par construction à chaque batterie.
- Fait : **parcours de première utilisation** — l'aide (`F1`,
  `src/ui/aide.rs`) porte maintenant les deux sections qui manquaient :
  **« Premiers pas »** (créer un projet, écrire, ajouter une source,
  exporter — les quatre gestes du premier quart d'heure) et **« Vos
  données »** (le chemin des données **visible**, plus un bouton « Ouvrir
  le dossier des données » → `ouvrir_chemin`). Épreuve
  `l_aide_se_bascule_et_dit_le_chemin_des_donnees` : la fenêtre se bascule
  et le chemin montré est le vrai dossier des données. Les erreurs
  actionnables du premier usage étaient déjà couvertes (création : nom
  vide, nom pris — les tests disent le geste).
- Fait : **le paquet Linux** — `scripts/paquet-linux.sh` fabrique
  `dist/typst-writer-<version>-linux-x86_64.tar.gz` (+ `.sha256`) :
  binaire release, entrée de menu (sans icône inventée), docs,
  `install.sh`, `desinstaller.sh`, et **`inventaire.txt`** (chaque fichier
  avec son empreinte). L'installation est un geste sans sudo
  (`~/.local/…`), la **mise à jour** = réexécution (remplacement propre,
  pas de téléchargement automatique), la **désinstallation** retire
  exactement ce qui a été posé et laisse les documents intacts.
  **Intégrité** : `sha256sum --check --strict` avant toute copie —
  éprouvé en ennemi : un fichier altéré est refusé, une ligne d'inventaire
  malformée est refusée (le `--strict` manquait, trouvé par le test), et
  le cycle complet (installation → mise à jour → désinstallation → piste
  propre) passe en HOME jetable. **Le binaire n'a jamais été lancé** pour
  vérifier : `ldd` (0 bibliothèque manquante), empreintes et inventaire
  suffisent. Doc : `docs/guide/installation-linux.md`.
- À faire (R7) : la CI et les essais natifs Windows/macOS sont à faire
  chez l'utilisateur (pushs distants interdits ici, machine Linux
  unique) ; paquets Windows/macOS à leur suite.

## R1 — coffre-fort des documents : TERMINÉ (release `0.3.0-alpha.3`)

- Clos : le coffre-fort est livré et **publié** (`v0.3.0-alpha.3`) —
  écritures atomiques, verrou natif, sauvegardes/conflits, corbeille,
  récupération des temporaires perdus. Les lignes ci-dessous en détaillent
  les pièces ; l'acceptation R1 (durabilité sous panne réelle) reste une
  épreuve de machine, hors de cette plateforme.

- Fait : les deux tests qui ouvraient des projets personnels utilisent des fixtures synthétiques. Le binaire des tests est lancé avec HOME et dossiers XDG jetables.
- Fait : au démarrage, le verrou est demandé avant la migration vers `document.tw` ; une seconde instance sans verrou ne déclenche pas cette migration. Test d'intégration isolé : échec reproduit avant correction, succès après.
- Fait : la migration de fichiers anciens à plat ne se déclenche plus lorsque des projets existent déjà. Test d'intégration isolé : échec reproduit avant correction, succès après.
- Fait : les écritures texte et binaires utilisent un fichier temporaire créé exclusivement avec un nom propre à l'opération, synchronisé avant remplacement ; les permissions existantes sont conservées. L'ancien `.tmp` n'est plus écrasé. Tests : ancien temporaire (rouge avant correction), écritures concurrentes et permissions Unix.
- Fait : l'épreuve d'espace ignorée a été mise en cohérence avec le geste d'ouverture dans un nouvel onglet : `NewProject` remplace désormais la vue courante. Les trois vues et leur restauration sont exercées par l'ouverture explicite avec Maj ; elle passe en HOME jetable.
- Fait : un fichier de verrou vide, invalide ou illisible ne peut plus être repris comme si son propriétaire était mort. La création d'un verrou signale une erreur d'écriture au lieu de l'ignorer. Test isolé : échec reproduit avant correction, succès après. Un fichier incomplet laissé après crash nécessite encore un geste manuel ; la solution portable et sa récupération restent à concevoir.
- Fait : la demande de fermeture est interceptée et les vues modifiées sont sauvegardées avant l'ordre de fermeture ; un échec conserve le tampon et annule cet ordre. Épreuves isolées : fermeture de deux projets et erreur de remplacement. L'événement natif et le geste utilisateur ne sont pas validés par ces tests.
- Fait partiellement : en l'absence de verrou, la frappe et plusieurs mutations sont refusées, notamment la suppression de mot, de notice, l'import d'image et de fragment, l'application de normes, la création de chapitre et l'écriture de l'export Zotero. Les commandes de l'éditeur sont désormais classées exhaustivement. Les réponses destinées à un onglet remplacé sont ignorées, sans jeter son tampon non sauvegardé. L'inventaire de toutes les mutations, les exports et les autres actions asynchrones restent à vérifier.
- Fait partiellement : le renommage LSP vérifie avant toute écriture que ses cibles résolues appartiennent au projet courant et ne touchent aucun tampon modifié. Il conserve les anciens fichiers temporaires et signale une erreur d'écriture partielle. Des épreuves ont reproduit l'écrasement d'un projet étranger, la perte d'un tampon et la suppression d'un ancien temporaire avant correction. Restent notamment la cohérence des autres vues sur des documents ouverts, la péremption des aperçus et l'atomicité globale multi-fichiers.
- Fait partiellement : supprimer ou renommer un chapitre contrôle son dossier et celui de l'assemblage avant toute écriture. Un test a reproduit le déplacement d'un chapitre d'un autre projet ; deux tests isolés vérifient maintenant le refus de suppression et de renommage interprojet, ainsi que la conservation de l'assemblage courant.
- Fait partiellement : `SaveAs`, sans entrée d'interface identifiée, choisit une destination avant d'écrire une copie du texte courant. Il ne change plus le chemin du document ni son état de sauvegarde ; le format `.tw` ne reçoit plus l'extension `.typ`. Une épreuve a reproduit le changement de chemin avant correction ; elle vérifie désormais que les frappes restent à sauvegarder. Des épreuves vérifient aussi le refus d'un document ouvert et d'un projet connu dont le verrou n'est pas détenu. Les projets extérieurs inconnus et les courses restent à examiner.
- Fait partiellement : les exports PDF et de mise en page ne réalisent plus l'écriture dans la tâche asynchrone ; le retour vérifie la destination avant remplacement atomique. Épreuves reproduisant puis empêchant l'écrasement dans un autre projet verrouillé ; un alias de dossier Unix et une réponse tardive à une vue remplacée sont aussi testés. Les exports hors projet ont été vérifiés, dont un PDF réellement compilé. Les courses, liens et plateformes non couverts restent à examiner.
- Fait partiellement : la destination d'export reconnaît maintenant un projet extérieur par la signature de l'appli (`document.tw`, marqueurs de verrou), même sans être listé ni verrouillé. Épreuve rouge reproduisant l'écrasement d'un export dans un tel projet, puis verte après correction. Une épreuve contraire épingle que l'export vers un dossier Typst ordinaire (`main.typ` seul) reste autorisé : un projet que l'appli ne peut pas ouvrir n'est pas réclamé par elle. Restent les projets hérités sans `document.tw` hors du dossier des projets.
- Fait partiellement : `RenderDone` compare désormais l'empreinte rapportée par la compilation à celle que la vue compilerait maintenant ; un retour périmé — frappes depuis la compilation, ou projet remplacé sous le même nom — est ignoré et la vue recompile au tick suivant. Épreuves rouges puis vertes : retour périmé simple, et suppression puis réutilisation du nom `mon-travail` avec le rendu de l'ancien projet en retard. Deux projets au contenu identique partagent la même empreinte : leurs rendus sont alors interchangeables, ce qui est exact.
- Fait partiellement : après sauvegarde et déplacement d'un projet, ses vues, ses onglets et ses sources sont retirés ; la suppression du dernier projet prépare un remplacement avant le déplacement. Une épreuve a reproduit des vues restantes avant correction ; avec plusieurs panneaux et réutilisation du nom, les retours `RetourVue` de ces vues sont ensuite ignorés. Les tâches qui écrivent directement hors du routeur restent à traiter.
- Fait partiellement : déplacement et suppression vérifient le verrou de la cible et sauvent les vues modifiées avant d'agir ; la suppression est annulée sur erreur. Après renommage, les vues, la source partagée et la session suivent le nouveau dossier. Épreuves isolées avec deux instances, plusieurs vues et erreurs d'écriture. La robustesse du verrou lui-même reste à traiter.
- Fait partiellement : importer une image extérieure réserve un nom libre sans écraser les images homonymes ; un dossier `images` résolu hors du projet est refusé. Le remplacement d'une image existante a été reproduit avant correction ; tests isolés avec collision et lien sortant.
- Fait partiellement : chaque remplacement conserve d'abord la version du disque dans `sauvegardes/` (vingt copies par document, espacées d'une minute ; la version d'un conflit est toujours conservée). Un changement extérieur n'est jamais écrasé en silence : la question est posée une fois, `Échap` refuse pour ce changement-ci seulement, et une sauvegarde impossible n'écrit rien. La procédure de restauration est documentée au guide. Épreuves rouges puis vertes : conservation de la version précédente, conflit non écrasé, résolution avec la version du disque récupérable, refus non répété, et nos propres écritures sans question. Restent les cas d'échec restants (interruption à chaque étape), le débordement des vingt copies à observer en usage, et un geste de restauration dans l'application.
- Fait partiellement : la création refuse un projet existant et publie le squelette complet d'un seul renommage (voir ci-dessous) ; les fichiers sont écrits sans remplacement sous le verrou. La migration à plat conserve ce verrou pendant le déplacement ; un test synthétique vérifie la migration et la conservation de l'original. Le test de 32 créations simultanées reproduisait plusieurs gagnants avant correction.
- Fait partiellement : un verrou natif porté par un fichier permanent sérialise les instances récentes ; l'identité de l'ancre est contrôlée après acquisition et le marqueur historique n'est supprimé que s'il porte notre PID. Le test de reprise concurrente, rouge avec 10 détenteurs auparavant, est maintenant actif et réussit sous Linux ; un autre test utilise deux processus et tue le détenteur. Le verrou est explicitement libéré même lorsqu'un descripteur a été dupliqué. Les anciens binaires ne respectent pas l'ancre ; leur coexistence reste dangereuse. Sur les autres systèmes, un PID historique est considéré vivant par prudence et sa récupération nécessite encore un geste manuel.
- À faire : essais natifs Windows/macOS, récupération assistée des marqueurs hérités, course avec anciens binaires et verrouillage pendant la migration des données à plat ; contrôle de toutes les mutations. Les correctifs actuels ne prouvent pas ces propriétés.
- Fait partiellement : une copie de sécurité impossible ou une écriture refusée ne remplace jamais le document, garde le texte en mémoire et dit son remède ; la version précédente est déjà conservée quand l'écriture échoue. Un temporaire abandonné par une panne devient, à la réouverture, une copie de `sauvegardes/` datée de son écriture ; un temporaire dont l'auteur est vivant n'est jamais touché. Les copies d'une même seconde ne s'écrasent plus et le renommage est synchronisé pour survivre à une coupure. Épreuves rouges puis vertes pour le temporaire abandonné ; les autres épinglent le comportement. Restent la vérification d'une vraie coupure de courant et le remplacement Windows.
- Fait partiellement : la création publie un projet d'un seul renommage, après avoir préparé le squelette complet dans un temporaire caché que le balayage ignore ; jamais un demi-projet en vitrine. La publication reste exclusive — l'épreuve des 32 créations simultanées vérifie qu'un seul gagne — et un travail interrompu part à la corbeille avec son auteur mort, jamais nettoyé en silence. Épreuves : balayage des dossiers cachés (rouge puis vert), squelette prêt à la publication, temporaire abandonné à la corbeille, création en cours intacte. Reste la coexistence avec les anciens binaires.
- Contrat de release R1 satisfait pour `0.3.0-alpha.3` : formatage régularisé dans son lot séparé (`cargo fmt --check` vert), tests du vendor entièrement verts (892) après comparaison avec l'amont pristine — une régression de la copie vendorisée, le défaut de `search_replace_enabled` inversé qui cassait deux tests vim, est réparée à la racine : le défaut reprend l'amont et l'application désactive sa recherche par l'API publique, documenté dans `PATCHES.md` ; `cargo deny check` propre (advisories, bans, licences, sources) ; paquet vérifié `dist/typst-writer-0.3.0-alpha.3-linux-x86_64.tar.gz` avec empreinte sha256, binaire ne dépendant que de libc selon `ldd`, `LICENSE`, `README`, `CHANGELOG` et le guide.
- Vérifié pour ce lot : build et clippy sans avertissement ; `cargo fmt --check` et `git diff --check` verts ; 405 tests ordinaires réussis, 78 ignorés, dont 60 épreuves d'espace exécutées séparément avec HOME jetable ; tests du vendor verts ; un PDF a été compilé avec le moteur Typst intégré. Le paquet de publication est produit sans lancer l'application — un lancement accidentel de `--version` a ouvert la vraie session de l'utilisateur (session.json réécrite, aucun document touché) et a été signalé.

## R2 à R9 : À FAIRE

Voir les livrables et critères dans la feuille de route ; aucun n'est déclaré livré par anticipation.

## bug-report-8 : les 20 tâches du panneau Normes (2026-11-04)

Les neuf chantiers du rapport `agent/bug-report-8.md` sont traités, avec le
compte rendu par tâche (cause, fichiers, tests, restes) consigné **en fin du
rapport lui-même**. Bilan : 512 tests ordinaires + 118 épreuves verts
(épreuves lancées une par une dans un HOME jetable, protocole oblige) ;
toutes les portes vertes. En bref : polices système et suggestions (T1),
quick parts vivantes partout (T4.1), couleurs unifiées pastille→roue (T6),
grille des titres couleur/italique/souligné (T2), images d'en-tête avec
sélecteur natif et hauteur en % (T4.2-4.4), guillemets à préréglages et
aperçu à trois niveaux (T5), formats de dates documentés et noms localisés
fr/en/de/it (T3), aperçu CSL fidèle — styles visibles — (T7), composants
harmonisés, persistance éprouvée et document de synthèse composé au vrai
Typst (T8-T9). Deux courses parallèles du cœur (corbeille, atelier d'import)
trouvées en chemin et réglées à la racine. La vérification visuelle du rendu
reste à l'auteur, par sa règle.

## Le glossaire glossarium (2026-11-04)

Fait et vérifié : le glossaire se gère entièrement du panneau gauche
(section « Glossaire » : fiche à 7 champs, clic = insertion au curseur,
crayon/corbeille au survol, recherche sur sa loupe). Le rendu est celui
du paquet **glossarium 0.5.10**, vendé dans `vendor/glossarium/` et
installé localement (`@local/glossarium`) à l'assemblage — jamais de
réseau. Balises `.tw` : `{glo:clé}` (+ `pluriel`, `majuscule`), liste
`{glossaire}` / `{glossaire: tout}` ; sur un `.typ`, l'insertion écrit
`#gls("clé")`. Données : `glossaire.toml`, ordre de l'auteur ; le `.tw`
unique en porte la section (clôture version 2, qu'une ancienne
application refuse proprement). Garanties éprouvées : clé inconnue →
`[clé]` + diagnostic, terme sans liste → sans lien (l'ancre de
glossarium manque sinon), le document compile toujours — prouvé au
**vrai moteur** (CLI Typst + pdftotext, rendu observé : « Le docteur
scolastique (sdo) d'abord, les sdo ensuite », liste « sdo – docteur
scolastique: … »).

Bilan de vérification : **527 tests ordinaires + 119 épreuves** verts
(épreuves à un HOME jetable chacune, batterie canonique) ; portes fmt /
test --no-run / clippy -D warnings / build / git diff --check / cargo
deny toutes vertes. Guide (`docs/guide/glossaire.md`, sommaire,
`ecrire.md`) et CHANGELOG à jour.

À faire : l'acceptation visuelle du panneau et de la fiche — elle
revient à l'auteur, par sa règle.

Leçon de lot : un script de substitution qui reprend un bloc existant
dans son ancre doit le **réémettre** dans son remplacement — la ligne
`let main = main_corrige;` avalée par un tel script jetait le fruit de
la guérison des citations, et seul le test multi-lignes l'a vu
(`un_document_ne_compile_jamais_mal_quoi_qu_il_arrive`).

## bug-report-9 T1.1 — les Normes prennent les mains (2026-11-05)

Fait et vérifié : à l'ouverture du panneau Normes, l'éditeur perd le
focus (`lose_focus`) — plus aucune frappe ni aucun geste de couleur ne
parvient au document pendant que le panneau est ouvert. Épreuve
`les_normes_prennent_les_mains`, **prouvée par mutation** (le correctif
retiré rougit l'épreuve, restauré la reverdit).

Bilan : portes vertes, 528 tests ordinaires à la clôture.

## bug-report-9 T1.2 + T1.4 — les normes se rendent au moteur (2026-11-05)

Cause trouvée, deux mécanismes distincts, tous deux prouvés au **vrai
moteur** (chaîne complète appliquer → assembler → composer) :

1. **Le gras ne pouvait pas se désactiver** : le gabarit n'écrivait
   `weight: "bold"` que dans le sens allumé — le gras par défaut des
   titres Typst survivait à la case décochée. Il s'écrit maintenant dans
   les deux sens (`weight: "regular"`).
2. **La police ne changeait jamais** : la typographie des normes était
   un `set text` posé par la fonction de mise en page, alors que
   l'assemblage porte le `#set text(font: …, size: …)` du gabarit
   d'origine — placé **après** dans le fil, il l'emportait. Sémantique
   Typst sondée à l'expérience (quatre compilations CLI) : la règle
   posée sur les **trames de texte** gagne toujours, y compris contre un
   réglage de titre posé à plat — les réglages de titre sont donc posés
   **en dedans** (`show heading.where(level: N): it => { show text: set
   text(…); it }`), seul endroit qui l'emporte sur la typographie du
   texte sans écraser les tailles de titres.

Modifié : `src/mise_en_page/gabarit.rs` (typographie souveraine, gras
bidirectionnel, réglages de titres en dedans),
`src/langage/pipeline.rs` (la sonde est devenue le test permanent
`les_normes_se_rendent_au_moteur` : `les_styles_de_titres_se_rendent_au_moteur`
+ `la_police_des_normes_l_emporte_sur_l_assemblage`, assemblage hostile
compris — gabarit d'origine avec son `#set text`).

Bilan : **529 tests ordinaires**, portes fmt / clippy -D warnings /
build / test / git diff --check / cargo deny toutes vertes ; la police
est prouvée par mutation (correctif retiré → rouge, restauré → vert).
Les épreuves applicatives (batterie canonique) courent en complément.

Reste : l'observation dans l'app revient à l'auteur, par sa règle — le
gras, l'italique, le souligné et les couleurs étaient déjà portés par le
gabarit ; ce que la sonde avait isolé, c'était le gras désactivé et la
police, désormais prouvés au moteur.

## bug-report-9 T1.2 — complément : la langue du moteur (2026-11-05)

Ce qui suit complète et corrige l'entrée précédente : la règle posée sur
les trames gagne, oui — mais elle perd quelque chose en chemin. Les
chaînes que le moteur localise lui-même (le sommaire « Sommaire », la
bibliographie « Bibliographie ») se lisent à la **portée d'élément**, où
un `show text: set text(lang: …)` ne les rend pas : les deux courses
d'acceptation (`le_devoir_synthetique_tient_ses_promesses`,
`la_these_synthetique_tient_ses_promesses`) l'ont vu le premier et
sont revenues avec « contents » et « bibliography ».

La typographie du texte s'écrit donc **deux fois** (`src/mise_en_page/gabarit.rs`,
`fn texte`) : `set text(…)` pour la portée d'élément (la langue, la
césure — ce que le moteur consulte avant de composer ses propres
textes), et `show text: set text(…)` pour la portée de trame (ce qui
l'emporte sur le `#set text` de l'assemblage). Les deux courses sont
vertes de nouveau, l'une et l'autre dans un HOME jetable.

Leçon : une règle qui « gagne toujours » gagne sur un front et peut
perdre sur un autre — la sémantique Typst se sonde par le produit fini
(texte extrait du PDF, chaînes localisées comprises), pas par le seul
style des trames.

## bug-report-9 T1.3 + T1.4 — le sélecteur de polices (2026-11-05)

Fait et vérifié : le choix de la police est un **vrai menu déroulant
rechercheable** (`iced::widget::combo_box`, natif — menu qui se pose
par-dessus, jamais de liste sous la barre) : la liste complète se
parcourt, la recherche la filtre (son filtre ignore casse, espaces et
signes — « timesnewroman » trouve « Times New Roman »), la saisie
directe est conservée (« une saisie valide se garde telle quelle »), et
l'aperçu montre la police choisie. L'ancien bloc de suggestions en
ligne, `familles_ressemblant` et `ressemblantes` partent avec lui et
leurs tests — la recherche vit désormais dans le widget.

T1.4 close en même temps : la police choisie atteint le moteur et
**l'emporte sur l'assemblage**, prouvé par
`la_police_des_normes_l_emporte_sur_l_assemblage` (mutation : correctif
retiré → rouge). La transmission par le champ est le chemin générique
des normes (`MiseEnPageChamp` → modèle → `appliquer`), éprouvé par la
persistance (bug-report-8, T8.3).

Modifié : `src/ui/reglages.rs` (`choix_police` reconstruit en ComboBox,
`corps_texte` reçoit l'état du menu), `src/app/state.rs` +
`src/app/mod.rs` (`police_combo`, la liste complète construite une fois
au démarrage), `src/typst/fonts.rs` (filtre maison retiré).

Bilan : **528 tests ordinaires** (un test du filtre retiré avec lui),
six portes vertes. L'aspect du menu dans l'app revient à l'auteur, par
sa règle.

## bug-report-9 T2.1 + T2.2 — le zoom migre à la barre supérieure (2026-11-05)

Fait et vérifié : les quatre commandes de zoom (« Réduire le zoom »,
« Ajuster à la largeur », « Ajuster à la hauteur », « Augmenter le
zoom ») vivent désormais dans le **segment central** de la barre
supérieure, avec leur pourcentage — au centre, comme demandé. La barre
de zoom de la fenêtre de rendu (les glyphes « − », « + », « W », « H »)
est supprimée : plus aucune duplication. Le groupe ne paraît qu'en mode
Rendu (TOPBAR-08 conservé : des commandes de zoom sans rendu visible
seraient des boutons morts).

Modifié : `src/ui/toolbar.rs` (le groupe centré, libellés conformes au
rapport), `src/ui/preview.rs` (l'ancienne barre et son commentaire —
qui justifiait l'ancien placement — partent ensemble ; `zoom_percent`
reste, la barre supérieure l'affiche).

Bilan : **528 tests ordinaires**, six portes vertes,
**120/120 épreuves** (batterie canonique, un HOME jetable chacune) à la
vérification précédente ; une batterie couronne l'ensemble en cours.
L'emplacement à l'écran revient à l'auteur, par sa règle.

## bug-report-9 T4 — proposition de lecteur PDF (2026-11-05)

Fait et vérifié : l'étude est livrée dans
`agent/proposition-lecteur-pdf.md` — **aucune implémentation** (la
tâche exige la proposition avant le code). Faits relevés à la source :
l'app rasterise aujourd'hui par `typst-render` (pas de dessin vectoriel
manuel), et la pixellisation au zoom vient du plafond
`MAX_PIXEL_PER_PT = 2.0` étiré — la netteté est donc une affaire de
rendu **par tuiles à la densité du zoom**, quel que soit le moteur.
Comparatif tenu : pdfium-render (PDFium, BSD-3) retenu ; MuPDF (AGPL)
et Poppler (GPL) écartés pour incompatibilité avec la licence MIT du
projet ; `pdf_render` (Apache-2.0/MIT, pur Rust) et hayro notés pour
plus tard. Architecture : un trait `MoteurPdf` à deux implémentations
(lecteur PDFium / chemin actuel en repli), le PDF exporté devient la
source d'affichage, étapes E1-E5 avec leurs preuves proposées.

Reste : la décision de l'auteur sur la proposition — le code n'avance
qu'après acceptation.

## bug-report-9 — le rendu vient du PDF, et la fenêtre ne rasterise que ce qu'elle montre (2026-11-05)

Fait et vérifié, en deux temps.

**1. pdf_render 1.3 devient LE moteur de rendu** (demande explicite de
l'auteur) : `render_page` interpète le **PDF exporté** (le vrai fichier
livré — l'aperçu et l'export ne font plus qu'un), `typst-render` est
retiré du projet. Les trois consommateurs sont branchés : pages de
l'aperçu, aperçu des titres, vignettes des projets. En chemin, la
vignette s'est trouvée corrigée à la racine : elle compilait le texte
« main.typ » lui-même (l'entrée d'un compilateur est du contenu, pas un
nom de fichier) — elle montre enfin la vraie première page
(`la_vignette_est_la_vraie_premiere_page`). La bascule éditeur → rendu
régénère systématiquement le PDF (`la_bascule_vers_le_rendu_genere_le_pdf`,
**prouvée par mutation**).

**2. La fluidité : le rendu par région et le geste différé.** Le zoom
était « lent » : mesuré, une page entière à 10 px/pt coûte **619 ms en
release** (et la molette en lançait jusqu'à quatre par cran). La
fenêtre ne montre pourtant qu'un douzième de la page. Le moteur est
donc vendu (`vendor/pdf-render-1.3.0`, `PATCHES.md`, deux entailles :
un champ `origin` neutre à `(0,0)`) et la fenêtre visible est
rasterisée **seule** — `render_region`, identique au rendu plein dans
le chevauchement (preuve `la_region_est_le_rendu_plein_recadre`, ±2/255
près, arrondi d'anti-crénelage). Mesure release, fenêtre 1200×900 px :

| densité | page entière | fenêtre visible |
|---|---|---|
| 1,5 | 31 ms | 39 ms |
| 5 | 94 ms | 26 ms |
| 10 | **619 ms** | **24 ms** |

Soit **27 fois plus vite au zoom maximal, et constant quel que soit le
zoom**. Le geste de zoom est différé : dix crans de molette ne
déclenchent qu'un rendu, quand le geste s'arrête
(`le_geste_de_zoom_ne_declenche_qu_un_rendu`, **prouvée par mutation**).
Défiler dans la marge de rendu ne coûte rien (couverture de tranche,
`tranche_couvre`).

Modifié : `src/typst/render.rs` (`render_page`/`render_region`/`ouvrir_pdf`
sur le PDF, `sonde_fluidite` = banc de mesure permanent),
`src/app/state.rs` (`Tranche`, `scroll`, `zoom_depuis`), `src/app/mod.rs`
(conducteur par région, geste différé sur `RenderTick`),
`src/ui/preview.rs` (offset de défilement remonté, région par page,
tests de géométrie), `src/project/mod.rs` (vignettes au PDF),
`src/mise_en_page/mod.rs` (aperçu des titres au PDF),
`vendor/pdf-render-1.3.0/` (le correctif `origin`).

Bilan : **532 tests ordinaires**, six portes vertes ; les épreuves
nouvelles prouvées par mutation. Reste, à l'ordre du jour : le zoom
ancré sur le curseur, puis la batterie canonique de couronnement.

Leçon de lot : un correctif de performance se mesure sur la chaîne
**livrée** — les chiffres de debug (1,9 s) et de release (619 ms)
racontent la même structure mais pas la même urgence ; et la première
épreuve du geste est tombée sur un vrai vide (aucune compilation livrée,
rien à rasteriser) — l'épreuve livre la réponse de la tâche, comme la
boucle de messages, avant d'affirmer quoi que ce soit.

## bug-report-9 — le geste de zoom : retour immédiat, ancre au curseur (2026-11-05)

L'auteur trouvait le zoom « encore très lent » malgré le rendu par
région. Trois mécanismes en cause, trouvés et réglés :

1. **Le retour visuel** : la tranche affichée gardait sa taille de rendu
   — au zoom, la page restait vide jusqu'à l'image nette. La tranche est
   maintenant une **fraction de la page** : elle s'étire avec le zoom,
   immédiatement, comme dans tout lecteur.
2. **L'attente du net** : le tic de rendu est à 300 ms et le geste
   différé de 200 ms — jusqu'à 500 ms de flou après le geste. Tic à
   100 ms, geste à 150 ms : le net arrive au plus ~250 ms après le
   dernier cran, le plus souvent moins.
3. **Le zoom s'ancre sur le curseur** (demandé) : le point sous la
   souris ne bouge plus — le défilement cible est **calculé puis imposé**
   (`set_preview_zoom`), ce qui neutralise au passage le défilement que
   ctrl+molette laissait fuiter : la molette ne fait plus défiler pendant
   le zoom. Sans curseur connu, l'ancre est le milieu de la fenêtre.

Modifié : `src/app/state.rs` (tranches en fractions, `fenetre` du
scrollable), `src/app/mod.rs` (ancre du zoom, tic à 100 ms, driver en
fractions), `src/ui/preview.rs` (étirement des tranches, régions en
fractions, epsilon de couverture), `src/app/message.rs` + `src/ui/layout.rs`
(le rectangle de fenêtre voyage).

Bilan : **532 tests ordinaires**, six portes vertes, épreuve du geste
verte. L'appréciation de la fluidité revient à l'auteur, par sa règle.

## bug-report-9 — la molette dirigée (2026-11-05)

Malgré l'ancre du zoom, ctrl+molette faisait encore défiler : le
défilement naissait **dans le scrollable**, avant que le message de zoom
ne puisse le rattraper — l'écrasement après-coup ne pouvait pas suffire.
Le défilement est donc désacréé **à la source** : un emballage de widget
(`MoletteDirigee`, `src/ui/preview.rs`) **consomme** l'événement de
molette selon les touches —

- molette simple : défilement vertical, comme avant ;
- **Maj + molette : défilement horizontal** (gauche-droite) ;
- **Ctrl + molette : le zoom seul**, ancré sur le curseur — le
  scrollable ne reçoit jamais l'événement.

La décision est une fonction pure (`geste_de_la_molette`), éprouvée ;
l'emballage délègue tout le reste (mise en page, dessin, `operate` —
le défilement programmé de l'ancre passe par lui) à l'enfant.

Modifié : `src/ui/preview.rs` (l'emballage, sa preuve),
`src/app/message.rs` (`MoletteHorizontale`), `src/app/mod.rs` (le
défilement horizontal s'exécute, `PAS_DEFILEMENT`).

Bilan : **533 tests ordinaires**, six portes vertes. La tenue en main
revient à l'auteur, par sa règle.

## bug-report-9 — la cible des défilements programmés (2026-11-05)

Deux signalements de l'auteur, une cause commune trouvée : **les
défilements programmés visaient l'identifiant nu du scrollable**
(`apercu`), alors que chaque vue monte le sien (`apercu-{id}`) —
`scroll_to` ne trouvait personne. Le zoom s'ancre donc en haut à gauche
(pas de déplacement), et le défilement horizontal ne suivait pas.

Second défaut, adjacent : **iced gère déjà `Maj`+molette en défilement
horizontal** (inversion d'axes native du scrollable) — l'emballage
interceptait `Maj` et cassait ce fonctionnement. Restauré : seul `Ctrl`
est intercepté (le zoom, sans défilement), `Maj` revient au scrollable.

Troisième, au passage : les sondes de taille du panneau
(`on_show`/`on_resize`) rapportaient le rectangle de fenêtre **à
l'origine** — une ancre de zoom sur cette origine était fausse. Le
rectangle de fenêtre n'est plus accepté que d'un vrai défilement.

Modifié : `src/app/mod.rs` (`id_du_scrollable` — la cible des deux
défilements programmés, le zoom et le suivi de page ; garde de la
fenêtre), `src/ui/preview.rs` (seul `Ctrl` intercepté, `Maj` rendu au
scrollable), `src/app/message.rs` (`MoletteHorizontale` retiré — le
scrollable le fait nativement).

Bilan : **533 tests ordinaires**, six portes vertes. La tenue en main
revient à l'auteur, par sa règle.

## bug-report-9 — l'initialisation du rendu, et le panneau des normes (2026-11-05)

Quatre demandes de l'auteur, traitées en deux causes.

**1. Le rendu vide au démarrage.** Le PDF de l'aperçu n'était fabriqué
qu'à la bascule de mode — jamais pour une fenêtre déjà ouverte. Ajouté :
un coup de pouce au tic (`rendre_les_fenetres_de_rendu`) qui donne à
**chaque fenêtre de rendu visible** — la vue active comme les onglets des
panneaux — l'ordre de montrer son document (`RenduInitialiser` : la
première compilation, puis le lecteur). Les résultats des tâches d'une
vue stationnée reviennent à leur vue (`etiqueter`), au lieu de se perdre
dans la vue active ; une tentative d'export par compilation interdit la
boucle sur un document en échec. Épreuve :
`au_demarrage_les_fenetres_de_rendu_se_remplissent`.

**2. Le panneau des normes.** Trois demandes, une seule mécanique — la
norme **chargée** est maintenant une notion d'état (`norme_originale`) :
- à l'ouverture, la norme de la bibliothèque **exactement égale** aux
  réglages du document est la norme chargée (`norme_qui_correspond`) ;
  sinon le panneau indique « Normes personnalisées » ;
- toute édition des réglages marque `modifie` : le panneau annonce
  « Modification de « X » » (ou « Normes personnalisées — modifiées ») ;
- l'enregistrement propose les deux gestes : **Écraser « X »** (la norme
  chargée), ou **Enregistrer une copie** sous un nom neuf — une copie
  refuse de voler un nom qui existe déjà, et devient la norme chargée.
  Supprimer la norme chargée rend le document « personnalisé ».

Modifié : `src/app/mod.rs` (coup de pouce, `RenduInitialiser`,
`NormesEcraser`, copie protégée), `src/app/message.rs`,
`src/app/state.rs` (`export_tente`), `src/app/espace_runtime.rs`
(résultats étiquetés), `src/app/reglages.rs` (détection, `modifie`),
`src/ui/reglages.rs` (état + deux enregistrements), `src/project/mod.rs`
(`norme_qui_correspond`, `correspondance`).

Bilan : **534 tests ordinaires** + deux épreuves nouvelles, six portes
vertes. L'appréciation revient à l'aateur, par sa règle.

## bug-report-9 — le panneau des normes épuré, et les couleurs de style (2026-11-05)

**Bogue réel, découvert par l'auteur : les couleurs de gras, italique et
liens ne changeaient rien au rendu.** Cause racine trouvée à la sonde :
les règles `show strong/emph/link: set text(fill:)` posées **à plat**
perdaient contre le `show text: set text(fill:)` de `fn texte` — tout
texte rendait `Rgb(0, 0, 0, 1)`, le noir du corps. Le correctif suit le
motif déjà éprouvé pour les titres (T1.4) : la règle est **imbriquée**
dans le contenu (`show strong: it => { show text: set text(...); it }`),
et l'emporte. Preuve permanente `les_couleurs_de_style_se_rendent_au_moteur`
(dans la chaîne réelle, valeurs observées à la sonde) : rouge, vert,
bleu aux trois styles. Au passage, la sonde a établi la syntaxe de la
langue : `**gras**`, `*italique*`, `[texte](cible)`.

**Le panneau, épuré** — un seul « Appliquer » (le menu applique en
choisissant), « Supprimer » devient l'icône poubelle du panneau
gauche (`icon_trash_2`), « Enregistrer une copie » devient l'icône
disquette (`icon_save`), et le texte de choix — l'état de la norme et le
geste « Écraser « X » » — descend sur **sa propre ligne**, sous le menu :
il n'écrase plus les autres éléments.

Modifié : `src/mise_en_page/gabarit.rs` (règles de couleurs imbriquées),
`src/langage/pipeline.rs` (preuve des couleurs), `src/ui/reglages.rs`
(icônes + ligne de choix).

Bilan : **535 tests ordinaires**, six portes vertes. L'appréciation
revient à l'auteur, par sa règle.

## bug-report-9 — le lien Zotero branche la bibliographie, et les sections en un geste (2026-11-05)

**Bogue signalé par l'auteur : une citation Zotero venait « introuvable ».**
Cause racine : le lien de collection écrit `references-zotero.bib`, alors
que le document citait `references.bib` — un fichier absent, qui casse la
compilation, et dont les clés Zotero ne vivent pas. Correctif de classe —
**le branchement des bibliothèques** (`brancher_les_bibliographies`,
`src/project/model.rs`) : l'appel `#bibliography` du document cite
exactement les bibliothèques qui existent (la locale, l'export Zotero,
celles déjà citées) — un fichier manquant ne l'est plus, la liste est
stable d'une compilation à l'autre. Il court aux deux bouts : à la
**liaison** (main.typ est écrit) et à la **chaque compilation** (les
projets déjà en souffrance guérissent d'eux-mêmes) — et la directive de
la section `{bibliographie}` se branche pareil.

**La langue : `{bibliographie}`.** La section bibliographie s'écrit
dans le texte comme `{glossaire}` — la liste **entière** du projet, à
cette place (parseur, schéma, sérialiseur, générateur).

**Trois boutons de la barre supérieure** : « Bib » insère une section
bibliographie, « Glo » une section glossaire — les deux dans le dialecte
du tampon (`{…}` en .tw, `#bibliography`/`#print-glossary` en .typ) — et
le menu « Mot du glossaire » insère `{glo:clé}` pour un terme déjà
référencé.

Modifié : `src/project/model.rs` (le branchement + ses preuves),
`src/langage/pipeline.rs` (branchement à la compilation, preuve Zotero),
`src/langage/parse.rs`, `serialiser.rs`, `schema.rs`, `generer.rs`
(`{bibliographie}` + preuve), `src/app/mod.rs` (branchement à la
liaison, listes), `src/app/message.rs`, `src/app/fichiers.rs`,
`src/ui/toolbar.rs`.

Bilan : **540 tests ordinaires**, six portes vertes. L'appréciation
revient à l'auteur, par sa règle.

## bug-report-9 — les schémas de numérotation libres (2026-11-05)

Demande de l'auteur : partout où le panneau des normes propose de la
numérotation, **son propre schéma doit pouvoir s'écrire**. Deux défauts
de classe trouvés :

1. **Le panneau** : sur les trois numérotations, les chapitres offraient
   déjà le duo modèle + saisie libre (bug-report-8, T2.3) — mais la
   **forme du numéro** (en-tête/pied) et la **numérotation des figures**
   n'avaient que des listes fermées. Les deux ont maintenant la saisie
   libre à côté du modèle, avec l'aperçu du schéma composé (« Aperçu :
   … ») — le défaut s'y montre comme « 1 ».

2. **Le moteur** : `forme_numerotation` (le gabarit) était une **liste
   blanche** — « I », « i », « A », « a », et tout le reste retombait en
   « 1 ». Un schéma libre s'y passe désormais tel quel — « 1.1 »,
   « 1) », « A.a » — c'est Typst qui compose. Les autres numérotations
   (chapitres, figures, marqueurs `{page}`) passaient déjà telles quelles.

Modifié : `src/ui/reglages.rs` (saisies libres + `apercu_ou_defaut`),
`src/mise_en_page/gabarit.rs` (passage libre), `src/langage/pipeline.rs`
(preuve : les schémas partent au moteur).

Bilan : **541 tests ordinaires**, six portes vertes. L'appréciation
revient à l'auteur, par sa règle.

## bug-report-10 — lots 1 à 3 : la barre et le glossaire (2026-11-05)

Trois lots du rapport 10, tracés au document d'état
`agent/IMPLEMENTATION_PROGRESS.md` :

- **Lot 1 (5.1, 5.2, 5.5)** : BIB/GLO en icônes, les styles de titre en
  icône ouvrant un menu (modèle du menu « Projet »), et une fine ligne
  de séparation entre les fenêtres (l'espacement du réseau de panneaux,
  1 px sur fond de bordure).
- **Lot 2 (5.4)** : le choix d'une couleur (« Enregistrer » — la
  dernière couleur, rien inséré) se distingue de son application
  (« Appliquer ») ; la pastille reste l'accès rapide à la balise de la
  dernière couleur. Épreuve `le_choix_de_couleur_n_insere_rien`.
- **Lot 3 (5.3)** : la fenêtre du glossaire (`glossaire_fenetre.rs`)
  réunit consulter, chercher, insérer au curseur, ajouter, modifier,
  supprimer — la liste déroulante de la barre devient son icône.
  Épreuve `la_fenetre_du_glossaire_s_ouvre_et_se_ferme`.

Bilan : **541 tests ordinaires**, six portes vertes. L'appréciation
revient à l'auteur, par sa règle.

## bug-report-10 — lot 4 : structure documentaire (2026-11-05)

- **1.1 / 1.2** : deux normes de placement (onglet Bibliographie) — un
  saut de page avant la bibliographie, et la bibliographie en titre de
  niveau 1 (`= Bibliographie`, la liste perdant son titre propre).
  Appliquées à l'assemblage (`placer_la_bibliographie`), éprouvées par
  `la_bibliographie_se_place_selon_les_normes`.
- **1.3** : la balise `[[TOC]]` de la langue `.tw` — table des matières
  composée par le moteur (`#outline()`) — avec son bouton dans la barre
  supérieure (dialecte respecté : `#outline()` en `.typ`). Éprouvée par
  `la_ligne_toc_est_un_bloc`.

Bilan : **543 tests ordinaires**, six portes vertes, **126/126
épreuves**. Le document d'état `agent/IMPLEMENTATION_PROGRESS.md`
continue le récit (les lots 5 à 8 ouverts : panneau gauche, profils de
métadonnées, listes multiniveaux, rendu).

## bug-report-10 — lot 5, partie 1 : le sommaire et la bibliographie (2026-11-05)

- **2.1** : les rangées du sommaire se touchent (colonne à espacement
  nul) — les lignes hiérarchiques dessinées sont continues, sans espace
  aux bordures du panneau ; l'indentation est conservée, les barres
  s'allongent d'une rangée à l'autre.
- **2.2** : chaque entrée de bibliographie se replie à part (chevron —
  repliée, elle se réduit à sa première ligne), et la recherche montre
  ses résultats même la section repliée.

Bilan : **543 tests ordinaires**, clippy vert. Reste au lot 5 : 2.3
(Projets + bibliothèque), puis les lots 6 à 8 — le document d'état
`agent/IMPLEMENTATION_PROGRESS.md` tient le récit.

## bug-report-10 — lot 5, partie 2 : Projets + bibliothèque (2026-11-05)

**2.3** — revérifié au code avant d'écrire (la règle du rapport d'état) :
la section Projets avait déjà les trois gestes (« Nouveau projet »,
crayon = renommer, poubelle = supprimer) et le choix d'étagère — la
demande décrivait un état déjà partiellement tenu. Ajouté : l'accès à
la **bibliothèque (vue étagères)** depuis la section, par un bouton à
icône (`ProjetsVue`, comme la barre supérieure).

Le lot 5 est complet. Bilan : **543 tests ordinaires**, clippy vert.
Restent les lots 6 à 8 — le document d'état tient le récit.

## bug-report-10 — lot 6, fondation : profils de métadonnées (2026-11-05)

La couche de données et ses traitements sont faits et éprouvés :

- **4.1 / 4.2** — les profils nommés (`Profil { nom, paires }`) portent
  chacun leurs valeurs ; le même mot y vaut différemment (« auteur :
  Jean / Jacques », éprouvé). Le profil actif est celui que les balises
  lisent (**4.3**, par `entrees()` qui alimente `sys.inputs`).
- **4.4** — `remplacer_les_balises` remplace les balises `{clé}` du
  texte par leurs valeurs ; une balise inconnue garde ses accolades.
  Le geste `MetaRemplacer` l'applique au tampon (`replace_span`).
- **4.5** — `cle_reservee` : `date`, `glo`, `glossaire`, `graphique`,
  `bibliographie`, `couleur`, `toc`, `titre` sont réservés par la
  langue ; `MetaChamp` refuse une clé réservée avec un message.

Bilan : **546 tests ordinaires** (trois preuves neuves), clippy vert.
**Reste au lot 6** : le panneau (choisir / nommer / supprimer un profil,
et le bouton « Remplacer les balises ») — les messages sont prêts.

## bug-report-10 — lot 6 complet : profils de métadonnées (2026-11-05)

Le panneau rejoint la fondation de l'étape précédente : la rangée des
profils (menu des profils, nom à enregistrer, icônes enregistrer /
supprimer) et le bouton « Remplacer les balises du texte par leurs
valeurs ». Les 4.1–4.5 sont tenus : profils multiples et valeurs par
profil, balises sur le profil actif, remplacement automatique, noms
système réservés (refusés avec message).

Bilan : **546 tests ordinaires**, clippy vert. Restent les lots 7
(listes multiniveaux) et 8 (rendu) — le document d'état tient le récit.

## bug-report-10 — lot 7, partie 1 : les listes multiniveaux (langue) (2026-11-05)

**3.1 / 3.5** — un élément de liste porte son **niveau d'imbrication**
(`ElementListe { niveau, contenu }`, deux espaces de retrait par
niveau) : le parseur lit le retrait, le générateur écrit l'imbrication
Typst en retrait, le sérialiseur `.tw` la garde — preuve
`une_liste_multiniveau_se_lit`. C'est un changement de modèle voulu :
les tests qui énonçaient la liste plate ont été portés au nouveau type.

Un accroc de fabrication signalé et réparé : une insertion de test mal
échappée a corrompu les littéraux de `parse.rs` — le bloc fautif est
retiré par numéros de ligne, puis la preuve repose proprement (script
en fichier, assemblage par lignes). 547 tests verts, clippy vert.

**Reste au lot 7** : les gestes de l'éditeur (3.2, 3.3, 3.4, 3.6).

## bug-report-10 — lot 7, partie 2 : les gestes de liste (2026-11-05)

**3.2 / 3.3 / 3.6** — sur une ligne de liste (le plan de touches le sait
par `Situation::sur_liste`) : `Tabulation` augmente le niveau,
`Maj+Tab` ou `Ctrl+Tab` le diminue — et `Ctrl+Maj+Haut/Bas` déplace
l'**élément de liste** (le déplacement de bloc reste aux autres lignes).
Le curseur suit sa ligne. Les aides sont pures et éprouvées
(`tests_des_gestes_de_liste`, 3 preuves).

Bilan : **550 tests ordinaires**, clippy vert. **Reste au lot 7** :
l'autocomplétion `1. ` et `Entrée` qui enchaîne (3.4) — puis le lot 8.

## bug-report-10 — lot 7 complet : la suite des listes (2026-11-05)

**3.4** — `Entrée` sur un élément de liste crée l'élément suivant : la
touche a déjà posé son saut chez le widget, le geste n'ajoute que la
marque suivante — la puce reste, le numéro avance (`3. ` → `4. `), et
hors liste rien ne s'enchaîne. Preuve
`la_suite_de_la_liste_se_continue`. Le lot 7 (3.1–3.6) est complet.

Bilan : **551 tests ordinaires**, clippy vert. Ne reste du rapport 10
que le **lot 8** (6.1 bibliographies dupliquées, 6.2 synchro édition /
rendu).

## bug-report-10 — lot 8, partie 1 : les bibliographies dupliquées (6.1) (2026-11-05)

**Cause établie par sonde** (deux compilations Typst réelles) : un appel
qui fond deux `.bib` partageant une clé lève `duplicate bibliography
keys` — c'est exactement l'erreur de l'utilisateur (le mélange
bibliothèque locale + export Zotero). Et la guérison des bibliographies
ne balayait que le **premier** chemin entre guillemets d'un appel
multiple : le second fichier n'était jamais guéri, donc jamais dédoublonné.

**Deux correctifs de classe** : le balayage cite désormais **tous** les
chemins de l'appel ; et la guérison retire les entrées dupliquées en
gardant la première bibliothèque — la locale gagne sur l'export Zotero,
qui reste en lecture seule. Preuve par le vrai moteur
(`les_cles_dupliquees_ne_cassent_plus_le_rendu`) : le document à clé
partagée compile sans erreur.

Bilan : **552 tests ordinaires**, clippy vert. **Reste au lot 8** : la
synchro édition / rendu (6.2) — le dernier point du rapport 10.

## bug-report-10 — lot 8 complet : la synchro édition / rendu (6.2) (2026-11-05)

**Cause lue dans le mécanisme** : la compilation n'attendait que le
repos de 2 s — l'horloge se repoussant à chaque frappe, **taper sans
s'arrêter ne rendait jamais** ; et basculer en vue Rendu montrait la
dernière compilation, pas le texte courant.

**Deux correctifs** : **le battement** (au bout de 2,5 s de frappe
continue, le résultat paraît quand même) et **le basculement qui compile
d'entrée** quand le rendu est périmé. L'ancienne règle d'épreuve « la
frappe ne compile jamais » est remplacée par sa forme exacte : jamais
une compilation par frappe — seul le battement montre le texte en
cours.

**Rapport 10 complet** : lots 1 à 8. Bilan final : **553 tests
ordinaires**, batterie **126/126 épreuves**, clippy vert.

## Corrections de l'utilisateur après le rapport 10 (2026-11-06)

Trois demandes, prises dans l'ordre dit.

**1. La bibliographie du panneau gauche** — le repli par entrée
(`references_pliees`, le chevron de chaque ligne) est **retiré** : le
repli est celui de la section entière, et la recherche le perce
toujours (les entrées qui matchent se montrent même repliées — c'était
déjà le cas, c'est désormais la seule règle).

**2. Les déclencheurs de rendu** — la règle est celle dite :
**bascule de mode**, **arrêt de frappe de 1,5 s** (IDLE_DELAY_MS passe
de 2 000 à 1 500), **`Entrée`** (le saut de ligne). Déclencheurs non
exclusifs. Le « battement » inventé la veille est **retiré** avec son
épreuve ; l'épreuve de la règle redevient `typing_never_triggers_a_compile`
(la frappe ne compile jamais elle-même).

**3. Un seul PDF par fichier** — les fenêtres de rendu tenaient chacune
leur propre compilation et leur propre PDF. Un registre partagé
(`rendus_partages`, par empreinte de texte) tient le document compilé et
le lecteur PDF en `Arc` : la compilation et l'export nourrissent le
registre, et toute fenêtre qui montre un texte déjà rendu **puise dedans
avant toute compilation** — `Arc::ptr_eq` prouve que c'est la même
pièce, jamais une copie (preuve `les_pieces_du_rendu_se_partagent`).

Bilan : **553 tests ordinaires**, batterie **126/126 épreuves**, clippy
vert. Acceptations visuelles laissées à l'utilisateur.
