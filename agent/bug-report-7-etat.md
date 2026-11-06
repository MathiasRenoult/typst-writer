# bug-report-7 — état de traitement (2026-11-02)

Traitement par lots, complexité croissante. Chaque point dit ce qui a été
fait, ou ce qu'il demande encore. Les numéros sont ceux du rapport.

## Faits et vérifiés

- **5 — siècles en petites capitales** : la racine était trouvée — le
  `#smallcaps[XX]` était écrit **avant** l'échappement Typst, donc tué dans
  le rendu. Règle neuve : la capitale et le marqueur **en exposant**
  (`#smallcaps[XIX]#super[e]`), écrits après l'échappement (marqueurs en
  caractères de contrôle, comme les quick parts). Déclencheur resserré
  (romain en début de mot seulement), et le backtique reste la déclaration
  explicite qui coupe tout. 4 tests (`tests_siecles`).
- **7 — bouton étiquette** : il refusait le `.tw` faute de syntaxe —
  périmé : `<nom>` est lu comme étiquette depuis R5 (et un renvoi
  `@fig:nom` la trouve). Le geste écrit `<>` dans les deux dialectes,
  curseur dedans. Épreuve `le_bouton_etiquette_pose_une_etiquette` ;
  l'ancienne épreuve qui énonçait le refus réécrite sur la règle neuve.
- **16 — boutons de format (puces, numérotée, citation, titres)** : la
  panne était réelle — sur un **bloc vide** (projets neufs, modèles),
  `formater_le_bloc` ne faisait rien en silence. Un bloc vide se forme
  maintenant : la marque s'écrit (`- `, `1. `, `> `, `# `), le curseur la
  suit. 4 épreuves `un_bloc_vide_se_formate_en_*` + l'épreuve du cas plein.
- **18 (bis) — bouton « Ouvrir le document d'écriture »** : il ne faisait
  rien en silence quand le document était déjà actif (l'air d'un bouton
  mort). Il répond maintenant (« Le document d'écriture est déjà sous les
  yeux. ») et ramène au document depuis un chapitre. Épreuve
  `le_bouton_document_ecriture_agit_toujours`.
- **22 — bouton « Accrocher »** : intitulé réécrit pour se comprendre :
  « Écrire dans main.typ le rappel des normes : le document se compose
  alors seul, hors de l'application, avec sa mise en page ».
- **6 — refresh du rendu à la bascule** : `TogglePreview` lance la
  compilation quand l'aperçu reparaît — sans purge (double tampon
  conservée, pas de clignotement). L'effet visuel est à vérifier dans
  l'app.

## À faire — par lot, avec ce qu'ils demandent

### Lot barre haute (1)

- **1 — barre haute contextuelle** : **FAIT** — la barre suit
  `state.mode` (Édition/Rendu/Typst, Ctrl+E) : en **rendu**, un groupe de
  zoom apparaît (réduire, ajuster à la largeur, ajuster à la hauteur,
  grossir — les messages `PreviewZoom*` existaient déjà) ; les outils
  d'édition (gras, italique, titres, listes, citation, couleur) n'y sont
  plus. Les communs (normes, recherche, projet, export) restent partout.
  Vérification visuelle à votre main.

### Lot bibliothèque (12, 23)

- **12 — aperçus de la bibliothèque** : **FAIT** — diagnostic : le
  pipeline fonctionne (éprouvé sur trois fixtures : projet minimal,
  projet de modèle, projet cassé) mais un projet dont `main.typ` ne
  compile pas **passait en silence**, sans vignette — d'où les cartes
  « Aperçu indisponible ». Le secours est réel : quand `main.typ`
  échoue, la vignette montre le **document d'écriture composé** (l'entrée
  est remplacée par le `.tw` généré). Test `une_vignette_revient_pour_un_
  projet_qui_compile` couvre les trois cas. Si vos cartes restent muettes
  sur vos vrais projets, dites-le — le prochain pas sera une sonde de
  compilation à leur endroit.
- **23 — légendes de figures** : **FAIT** — trois normes vivent dans le
  dialogue (onglet titres) : **libellé de la légende** (« fig. »,
  « Figure », autre), **numérotation** (qui existait déjà mais n'était
  consommée nulle part — la racine), et **style** (gras/italique). Le
  gabarit émet `set figure(numbering, supplement)` et
  `show figure.caption: set text(...)` ; vide partout = défauts de
  Typst, rien ne bouge pour qui ne règle rien. Test
  `la_legende_se_regle_et_se_genere`.

### Lot interface (10, 11, 13, 14, 17, 20, 21, 19)

- **10 — color wheel** : **FAIT** — curseur en croix sur la roue et le
  carré (`mouse_interaction`), champ hex copiable/collable qui pilote la
  roue en retour (`PickerCouleurHex`, la saisie reste sous les doigts si
  elle n'est pas encore un hex), RGB en direct. Épreuve
  `la_roue_s_entend_avec_le_hex_et_les_rapides`.
- **17 — quick colors** : **FAIT** — palette fixe de 8 pastilles sous la
  roue (avec étiquettes) : un clic applique sans fermer, pour peindre à la
  suite. Même épreuve.
- **11 — la même roue dans l'éditeur de thème** : **FAIT** — la roue vise
  maintenant les champs du thème (`PickerCouleur.champ`) : le bouton de
  couleur d'un champ ouvre la roue, l'appliquer écrit le champ
  (`ThemeEditeurChamp`). L'ancien picker à curseurs du thème est retiré
  (remplacement demandé au point 11 — messages `ColorPickerR/G/B/Valider/
  Fermer`, état `ColorPicker` et overlay supprimés). Épreuve
  `la_roue_du_theme_vise_son_champ`.
- **13 — barres hiérarchiques** : **FAIT** — le système est repensé : les
  connecteurs étaient des **glyphes de boîte** (`│ └ ├`) dans une police
  proportionnelle — chaque ligne dérivait, d'où la diagonale de la
  capture. Ils sont maintenant de **vrais traits** (conteneurs d'un
  pixel) : un niveau = une colonne, toujours au même x, guides pleins là
  où la branche continue, coude demi-trait pour la dernière branche.
  Rendu à votre acceptation visuelle.
- **14 — boutons « Projet » vers un menu de la top bar** : **FAIT** — le
  menu déroulant « Projet » (`iced_aw::menu`) porte les six gestes dans la
  barre supérieure, et les boutons de la latérale sont **retirés** — le
  rapport le dit explicitement (« ne devraient pas être là… sans polluer
  la left bar »).
- **20/21 — champs des normes qui dépassent, alignement** : **FAIT** —
  la disposition est passée en deux colonnes de portions
  (`FillPortion(2/3)`) : le contrôle démarre toujours au même x
  (l'alignement) et se resserre avec la fenêtre (le dépassement) ; les
  champs à largeur fixe 260 sont en `Fill`.
- **19 — tab/shift-tab dans les normes** : **FAIT** (câblage) — la
  fenêtre des normes (`state.mise_en_page`) entre dans le contrat des
  fenêtres (`fenetre_ouverte`) : `Tabulation` circule par
  `widget::operation::focus_next`, `Maj+Tabulation` revient. Épreuve
  `les_normes_sont_une_fenetre_au_contrat_de_clavier`. Le ressenti
  champ-à-champ reste à votre acceptation visuelle (focusabilité réelle
  des contrôles).

### Point 18 — l'onglet projets par étagère : **FAIT**

La liste de gauche ne montre plus tous les projets automatiquement :
elle montre **l'étagère du projet courant** (on travaille avec les siens).
Un bouton bascule vers **« Tous »**, et une **barre de recherche** filtre
par nom. Règle extraite en fonction pure `projets_visibles`, éprouvée par
deux tests (`tests_filtre` : étagère, tous, recherche, non-filés
ensemble).

### Lot écriture (8, 9)

- **8 — modèles devoir/thèse** : **FAIT** — les canevas écrivaient leur
  contenu en **Typst brut dans `main.typ`** (d'où l'éditeur vide et le
  rendu peuplé). Règle neuve : le modèle écrit un **document.tw visible**
  (Introduction, Développement, Conclusion, Bibliographie — tout
  modifiable) ; les **normes** portent la page de titre (née des
  métadonnées), le **sommaire** et la numérotation des titres ;
  `main.typ` n'est plus qu'assemblage. Deux normes nouvelles
  (`page_de_titre`, `sommaire`). Les épreuves R5 qui promettaient dans le
  PDF ce que le Typst caché fabriquait sont réécrites sur les promesses
  visibles. Tests `un_modele_ecrit_ce_que_l_editeur_montre` et les deux
  `*_promet_ce_que_l_editeur_montre`.

- **15 — note de bas de page** : **FAIT** — le geste ouvre un petit
  panneau (`src/ui/note.rs`), la note s'y écrit, et la **seule marque**
  vient au curseur (`^[note]` en `.tw`, `#footnote[note]` en `.typ`) —
  plus jamais le texte de renvoi tapé dans la ligne. Le panneau entre au
  contrat des fenêtres (Tabulation, Échap). Épreuve
  `la_note_se_ecrit_dans_son_panneau_et_pose_sa_marque` ; l'ancienne
  épreuve du geste réécrite sur la règle neuve. Reste, en second temps :
  masquer le texte de la note **dans l'affichage** de l'éditeur (le
  repli du vendor — `folding.rs` — est la voie, avec `PATCHES.md`).
- **8 — modèles devoir/thèse** : remplacer les gabarits « éditeur vide »
  par des fichiers de normes + documents **dans l'éditeur** (le modèle
  écrit les fichiers, l'éditeur les montre). Décision de conception à
  entériner avec vous avant la refonte.
- **9 — métadonnées de document** : **FAIT** (cœur) — un onglet
  **Métadonnées** en tête des normes : une paire clé → valeur par ligne,
  la clé est le nom de la quick part ({titre}, {auteur}, ou ce que vous
  nommez), ajout/suppression de lignes. Les données s'écrivent dès le
  geste (`metadonnees.toml`), jamais perdues ni annulées. **Racine** :
  les entrées `sys.inputs` que les quick parts appellent n'étaient
  remplies par personne — elles le sont maintenant à chaque compilation.
  Base semée (titre/auteur/date) pour un projet neuf. Tests +
  épreuve `les_metadonnées_se_reglent_et_se_sauvent`. **Achèvement** :
  le dialogue de **création** propose titre et auteur, et le projet naît
  avec ses métadonnées — épreuve `la_creation_porte_les_metadonnées`.

### Lot architecture (3, 4, 2)

- **3 — un `.tw` unique transportable** : **jalon 1 FAIT** — le format
  est spécifié (`agent/Pré-Beta/format-tw-transportable-2026-11.md`) et
  implémenté (`src/langage/portable.rs`, `FichierPortable`) : le document
  reste en tête, les pièces (métadonnées, normes, bibliographie) suivent
  dans une clôture `~~~~ tw-portable:1` à sections `@nom`. Rétrocompatible
  (un `.tw` d'avant = document seul) et versionnée (un fichier du futur
  se lit mais ne s'écrase pas — `version_inconnue`). 4 tests de
  va-et-vient. **Jalon 2 FAIT** : `exporter_tw`/`importer_tw`
  (`src/project/portable.rs`) — le projet voyage dans un seul `.tw`
  (document + métadonnées + normes + bibliographie) et se reconstruit de
  l'autre côté : pièces à leur place, accroche posée, `main.typ` prêt à
  composer. Un fichier du futur s'y refuse (rien de ce qu'on ne voit pas
  ne se perd). Gestes au menu Projet : « Exporter en .tw unique… » et
  « Importer un .tw unique… ». Test `un_projet_va_et_vient_dans_un_seul_tw`.
  **Jalon 3 FAIT** : la clôture est **indentée** — c'est la règle de
  repli de l'éditeur (une région s'ouvre quand la ligne suivante
  descend) : elle se replie d'un clic, **sans rien toucher au vendor**.
  Lecture dé-indentée à l'enveloppe près (l'indentation interne des
  notices BibTeX se garde). Test `la_cloture_se_replie_par_indentation`.
  **Point 3 complet.**
- **4 — architecture paquets Typst + GUI cetz** : **jalons 1-2 FAITS**.
  (1) Les paquets se résolvent **localement** : `packages_dir()`
  (`~/.typst-writer/paquets/<ns>/<nom>/<version>/`, déroutable par
  `TYPST_WRITER_PAQUETS`) et `path_of` sert les `@local/…` sans réseau —
  test `un_paquet_local_se_resout` (un `#import "@local/demo:1.0.0"`
  compile). (2) Le trait d'extension **`ExtensionPaquet`** et son
  registre (`src/extensions/mod.rs`) : identité, paramètres, insertion —
  la première extension (ceTZ) écrit le code Typst à la place de
  l'écrivain, testée. Ajouter une extension = un type de plus au
  registre. **Jalon 3 FAIT** : le composeur d'extension
  (`src/ui/extension.rs`) — ses champs **naissent de `parametres()`**
  (une extension de plus = une GUI de plus, sans une ligne de vue),
  l'inscription « Insérer une extension… » au menu Projet, le code posé
  au curseur. Épreuve `le_composeur_ecrit_le_code_a_sa_place`.
  **Point 4 complet — bug-report-7.md est achevé.**
- **2 — évaluation « export PDF + lecteur intégré »** : la demande est une
  **évaluation** — produite ci-dessous.

## Évaluation — point 2 : export PDF + lecteur intégré vs rendu maison

**L'idée est bonne, et c'est la direction à prendre**, avec une nuance.

- Le rendu maison (compilation Typst → raster de pages) a un coût connu :
  c'est lui que les mesures ont montré dominant (81 ms avec 1 000 notices),
  et son scrolling/virtualisation est du code à maintenir.
- Un lecteur PDF intégré (pdf.js ou un rendu PDF natif) reprend des
  composants éprouvés : défilement, zoom, sélection, recherche, impression
  — des années de polish gratuites.
- La nuance : l'export PDF est **différé** (compile + écriture) alors que
  le rendu maison est **vivant** (il suit la frappe). Deux voies :
  a. PDF à la demande + lecteur : simple, robuste, mais l'aperçu retarde ;
  b. garder le rendu vivant POUR L'ÉDITION et donner le lecteur PDF pour
  la LECTURE du document fini (mode lecture).
- **Recommandation** : la voie b à court terme (le rendu vit, le lecteur
  PDF s'ajoute pour le mode lecture et l'impression), la voie a si les
  mesures de confort montrent que la latence du PDF gêne moins que les
  défauts du rendu maison. Le tout suppose un composant PDF iced — à
  évaluer côté dépendance (un `pdf.js` via web view ou un lecteur natif).
