# Journal des versions

Les versions sont classées de la plus récente à la plus ancienne. Chaque
entrée décrit les fonctionnalités et les corrections livrées avec la version
correspondante.

## 0.6.3 — les personnes citées, une garde nette et des gestes d'édition (2026-11-25)

- Le plugin **Personnes citées** tient un registre `personnes.toml` par
  projet. Le panneau d'insertion filtre les entrées au fil de la frappe,
  remplit tous les champs de la fiche et n'écrit que la clé dans la balise.
  La syntaxe `{{p clé}}` suit les formes de première mention et de reprise
  des normes ; `|simple`, `|complet`, `|alias`, `|auto` et les listes de
  champs (`nom+dates`, `simple+naissance+mort`, etc.) permettent de choisir
  exactement ce qui s'affiche. Un clic droit sur une balise la rouvre déjà
  remplie et la remplace au lieu d'en ajouter une seconde.
- Une balise de personne sans expression applique de nouveau les règles de
  première mention ; la compilation de `grand-oral` avec plusieurs mentions
  de Beale est couverte par une épreuve de non-régression.
- La page blanche qui précédait `{{garde}}` est supprimée : la bibliographie
  cachée ne réserve plus une page, et le branchement de la garde dans le
  gabarit est idempotent même si une ancienne version y a laissé ses lignes.
- `Ctrl+flèche` déplace le curseur d'un mot, `Ctrl+Maj+flèche` étend la
  sélection par mot, et `Ctrl+Retour arrière` / `Ctrl+Suppr` suppriment le
  mot précédent ou suivant. Les accents et les frontières UTF-8 sont
  respectés.
- Le menu du clic droit ordinaire s'ouvre au point exact du clic, dans la
  vue de rédaction, au lieu d'être décalé par la barre ou le panneau.
- Les menus de la barre du haut ont une largeur bornée et des lignes
  neutres : les cartes « Projet », « Insérer » et « Titres » ne prennent plus
  la couleur d'accent ni toute la largeur disponible.
- Un bouton `+` fixe après les onglets ouvre une nouvelle vue sur le
  document courant sans devoir faire défiler la barre d'onglets.
- Le rail des onglets des normes et des plugins défile au lieu d'écraser son
  dernier élément quand la fenêtre est courte.

## 0.6.2 — gérer les versions bibliques (2026-11-18)

- Les tableaux se naviguent et se construisent au clavier : `Tab` / `Maj+Tab`
  passent d'une cellule à l'autre, `Entrée` / `Maj+Entrée` à la même colonne
  d'une autre ligne. `Ctrl+Tab` insère une colonne vide à droite, `Ctrl+Entrée`
  une ligne vide en dessous, et `Ctrl+Maj+flèches` déplace la rangée ou la
  colonne. Le clic droit sur un en-tête fait tourner son alignement. Chaque
  déplacement réaligne le tableau courant, sans toucher aux autres tableaux
  ni au reste du document.
- `Ctrl+Suppr` supprime la ligne de données courante d'un tableau et
  `Maj+Suppr` sa colonne, sans toucher à l'en-tête ni à la dernière colonne.
- `Ctrl+B`, `Ctrl+I` et `Ctrl+U` mettent en forme la sélection, ou à défaut
  le mot sous, avant ou après le curseur. Taper `"`, `(`, `[` ou `{` sur une
  sélection explicite l'encadre avec la paire correspondante.
- Un clic droit sur une balise couleur, date, glossaire ou métadonnée ouvre
  son formulaire prérempli ; valider la remplace sans en créer une seconde.
- `{centre}`, `{haut}`, `{bas}`, `{gauche}`, `{droite}` et les positions de
  coin sont les formes courtes de `{position=…}` ; la source relue les
  préfère.
- Une garde suivie de trois lignes vides ne laisse plus de page blanche :
  le saut de page initial du corps est retiré quand la garde en pose déjà un.
- Les dernières lignes de l'éditeur ne restent plus invisibles après un
  redimensionnement, un changement de mode ou une ancienne position de
  défilement restaurée : le rendu utilise les mesures réelles de la vue et
  borne le défilement au contenu.
- Le plugin **Personnes citées** tient un registre par projet
  (`personnes.toml`), écrit `{{p clé}}` et compose `{{index-personnes}}`
  avec les pages, les initiales, les dates et la qualité. Aucun nom n'est
  deviné dans la prose ; le registre se gère dans sa fenêtre et son onglet
  de plugins. Les normes distinguent la forme de la première mention, des
  mentions suivantes et de l'index (nom complet, simple ou alias), et la
  forme des dates (naissance et mort, naissance, mort ou texte libre).
  Le panneau d'insertion filtre le registre au fil de la frappe, remplit
  tous les champs de la fiche et n'écrit que la clé dans la balise. Sans
  expression, les formes des normes s'appliquent (nom simple par défaut ; le
  nom complet et les dates à la première mention se règlent dans le plugin).
- Le rail des onglets des normes et des plugins défile au lieu d'écraser
  son dernier élément quand la fenêtre est courte ; les noms longs peuvent
  passer à la ligne.
- Le curseur ne reste plus collé au bord haut ou bas de la fenêtre : un
  déplacement, un clic ou une frappe qui l'y amène déclenche le défilement
  pour le replacer dans la zone de lecture.
- La commande **Liste à puces** ne transforme plus le paragraphe entier :
  elle agit sur la ligne du curseur, ou sur toutes les lignes non vides de
  la sélection, et sur elles seules. La liste numérotée suit la même règle.
- Un triple clic sur une balise de tableau ne fait plus tomber
  l'application : le compteur de mots analysait la sélection comme un
  document complet et supposait une grille après la balise. Le parseur
  accepte désormais une balise seule.
- Les versions ajoutées apparaissent dans le panneau gauche avec leur nombre
  de livres, de chapitres et leur provenance : un clic choisit la version,
  **Dossier** ouvre le dossier local, la flèche copie le JSON pour le
  transporter, la corbeille retire une version vers la corbeille du système.
- Un écran **Restaurer une version** liste les copies de `sauvegardes/` avec
  date locale, taille, empreinte et aperçu, et restaure la copie choisie après
  confirmation ; le texte actuel est conservé avant le remplacement et le
  document affiché est réellement rechargé.
- Les versions bibliques s'installent hors du fil d'affichage ; un lot
  d'archives ne recharge le registre qu'une fois.
- Un bouton **modèle** enregistre un JSON minimal à remplir.
- Supprimer la version consultée remet la lecture sur Louis Segond sans
  laisser de référence orpheline.
- L'ajout ou le retrait d'une version ne décale plus les références déjà
  ouvertes : les versions ajoutées gardent leur indice stable ; le nom d'un
  livre cité suit la version qui le nomme.
- Le script `scripts/convertir-version-biblique.py` convertit un JSONL de
  versets ou un USFM simple vers le format local, avec les trous de
  numérotation préservés.
- L'export d'archive peut joindre les versions bibliques sur demande explicite,
  avec avertissement de droits, empreinte et restauration à l'import.
- La recherche peut parcourir tous les fichiers `.tw` et `.typ` du projet,
  afficher les résultats avec leur chemin et leur ligne, et ouvrir l'occurrence
  choisie ; le remplacement reste limité au document courant.
- Les erreurs de compilation sont soulignées dans l'éditeur et restent
  listées dans le panneau gauche quand l'aperçu est fermé ; un clic les ouvre
  à leur position dans la source.
- L'export DOCX rappelle avant le choix du fichier que les notes deviennent
  du texte, au lieu de ne le dire qu'après.
- L'index des citations sépare désormais deux traductions d'un même livre :
  une citation en Louis Segond et la même en Darby ne fusionnent plus sous
  une seule entrée.
- Les divisions de corpus de l'index sont des titres de niveau 2, et un
  titre de livre reste solidaire de sa première référence : il ne peut plus
  rester seul en bas d'une colonne.
- Les archives portables calculent une empreinte stable d'une exécution à
  l'autre ; une empreinte qui ne correspond pas est refusée avant toute
  installation.
- La recherche dans le projet attend la fin de la frappe (180 ms) et arrête
  un parcours devenu périmé avant de continuer à lire les fichiers.
- L'aperçu ne recompile plus pour produire son PDF quand le document compilé
  est déjà en mémoire ; sur un document de neuf pages, la compilation mesure
  ~30 ms et l'export PDF ~7 ms en release.
- Le rendu précharge une bande de la page voisine, garde le papier sous les
  textures et ne colle plus aucun texte de chargement sur les pages : un
  défilement rapide n'ouvre plus sur une page blanche ou un « Rendu… ».
  Les pages sont centrées horizontalement, les erreurs occupent toute la
  largeur du panneau, et les messages de compilation ou de PDF ne s'affichent
  plus au-dessus du document.
- Le planificateur de rasterisation sert d'abord les pages visibles, puis la
  page vers laquelle on va ; une génération de vue annule les tâches encore
  en attente pour une fenêtre quittée.
- **Aide → Corbeille et suppressions…** liste les entrées avec leur date,
  leur taille et leur origine, et les restaure à leur emplacement d'origine
  sans jamais écraser un fichier existant.
- La recherche du document n'ouvre plus la boîte de recherche propre à
  l'éditeur : une seule barre reste à l'écran. Elle n'est plus modale, on peut
  cliquer et faire défiler l'éditeur pendant qu'elle est ouverte ; « suivant »
  et « précédent » amènent réellement l'occurrence dans la fenêtre.
- Changer de mode ne renvoie plus au début : l'édition retrouve son défilement
  exact et le rendu sa position exacte. À la première ouverture du rendu, la
  page suit le haut de la vue d'édition.
- La **palette de commandes** couvre désormais les formats du texte (gras,
  italique, souligné, barré, gras italique, code, formules, titres, listes,
  citation, note, bloc Typst), les teintes orange/vert/rouge/bleu/violet/gris/
  noir et la dernière couleur choisie, ainsi que la roue des couleurs. Elle
  expose les imports et exports de fichiers, les thèmes, les normes de la
  bibliothèque (appliquer ou exporter), les bascules de plugins et les
  déplacements de blocs, titres et listes.
- Le projet n'a plus de vue par chapitre ni de fichiers de chapitres à gérer :
  `document.tw` est le seul document d'écriture, et le panneau gauche ne garde
  que le **Sommaire**.
- Le compte de mots exclut désormais les notes de bas de page ; une sélection
  affiche son propre compte dans la barre d'état. Le panneau donne aussi la
  part des citations de bloc (`>`), en mots et en pourcentage.
- Deux réglages de police prioritaire par script, « Hébreu » et « Grec »,
  permettent de composer l'hébreu avec Ezra SIL SR et le grec avec Galatia SIL
  sans changer la police du texte latin. Les deux champs sont vides par défaut.
- Les points de suspension ne sont plus suivis d'une espace dans le rendu.
- Les blocs de citation (`>`) ont leur apparence dans les normes : police,
  taille, couleur, retrait à gauche et italique ; vide = défaut de Typst.
- Dans le PDF, un clic simple ne sélectionne plus rien : la sélection ne
  commence qu'au clic suivi d'un mouvement ; double et triple clic gardent
  leur sens.
- Les icônes d'édition et de suppression d'une notice bibliographique ne
  paraissent qu'au survol de l'entrée, avec leur place réservée pour que la
  ligne ne se réorganise pas sous le pointeur.
- Les liens internes peuvent afficher la numérotation réelle du document :
  `[[titre-adoption-coutume|"chapitre"]]` donne « chapitre 2.3 »,
  `[[figure-planche|"figure"]]` donne « Figure 4 » et
  `[[titre-adoption-coutume|"page"]]` donne la page composée. Le numéro du
  chapitre suit la numérotation des titres, le préfixe de la figure suit le
  libellé des légendes, et la page suit la pagination. La forme courte
  `adoption-coutume` suffit quand elle est unique.
- Les **pages de garde** s'écrivent entre `{{garde}}` et `{{/garde}}` : elles
  précèdent le sommaire, ne portent aucun numéro et n'entrent pas dans la
  table des matières, même quand elles contiennent des titres. Plusieurs blocs
  donnent plusieurs pages ; le menu `,,` les insère et la page de titre des
  normes reste elle aussi sans numéro.
- La propriété `{position=…}` place un élément d'une page de garde : `haut`,
  `centre`, `bas`, `gauche`, `droite` et leurs coins. Une page de titre sans
  métadonnée n'est plus composée : plus de page blanche avant la première
  garde.
- Les citations collées au mot (`famille@grudem…`) et celles qui terminent un
  paragraphe sont reconnues comme citations Typst ; la typographie ne mange
  plus les espaces du paragraphe, et un guillemet fermant après une citation
  retrouve son insécable. Une adresse électronique reste du texte.
- La barre supérieure est réorganisée en trois domaines : à gauche le projet
  et le document, au centre les gestes d'écriture, à droite les normes, les
  plugins, les paramètres et l'apparence. Plus aucun bouton ne partage une
  icône avec une action différente.
- Les insertions se rangent dans un menu **Insérer** (sommaire, notes,
  citations, image, tableau, graphique, sections, bible, glossaire,
  extensions), au lieu d'une icône par outil. Le menu Projet ne porte plus
  que les gestes de fichier et d'archive.
- Une page **Paramètres** (`Ctrl+,`) règle l'application elle-même : elle peut
  masquer les boutons de mise en page pour qui écrit le markdown, masquer le
  menu d'insertion, afficher les numéros de ligne et choisir les thèmes. Ces
  réglages sont retenus dans la session, pas dans les normes du document.
- Les contrôles de thème et le bouton des numéros de ligne quittent la barre
  supérieure pour cette page ; la barre garde son rôle de navigation.
- Les tableaux peuvent recevoir un **titre** : `{{t "Taux"}}` au-dessus de la
  grille. Le titre devient une légende numérotée « Tableau 1 » avec son propre
  compteur, et le tableau est référençable par `[[tableau-taux]]` ou
  `[[tableau-taux|"tableau"]]` ; le panneau d'insertion demande le titre.
- Les références bibliques placées dans les cellules d'un tableau sont
  désormais comptées dans les statistiques et le relevé : une ouverture
  canonique comme `{{t}}` ne coupe plus le balayage des références suivantes.
- Un tableau sans titre reste référençable : il reçoit l'ancre de rang
  `tableau-1`, `tableau-2`… et n'affiche aucune légende. L'option
  `show-title=true` d'un lien (`[[taux show-title=true|le tableau]]`) affiche
  le titre de la table quand elle en a un ; sinon le texte après la barre
  reste le repli.
- La commande **Aligner les tableaux** égalise les colonnes sans toucher au
  titre ni au reste du texte. Dans une cellule, `Tab`/`Maj+Tab` passent d'une
  cellule à l'autre, `Entrée`/`Maj+Entrée` à la même colonne d'une autre
  ligne, et `Ctrl+Tab`/`Ctrl+Maj+Tab` restent des alias ; le tableau courant
  est réaligné à chaque déplacement.
- La section bibliographique n'est **jamais** ajoutée automatiquement : elle
  n'existe que si `{{bib}}` (ou `{{bibliographie}}`) figure dans le texte.
  Une ancienne directive `#bibliography(…)` restée dans `main.typ` est
  retirée de la composition, sans réserver de place : elle charge les notices
  pour les citations, n'affiche aucune section et ne modifie pas le fichier.
  Les chemins de bibliographie morts sont remplacés par les bibliothèques
  vivantes du projet.
- La barre d'état nomme le fichier de normes du document
  (`mise-en-page.toml`) et, quand les réglages correspondent exactement à
  une norme de la bibliothèque, son nom. Les panneaux **Normes** et
  **Plugins** se basculent sans perdre la copie de travail, et le panneau des
  plugins propose **Écraser « … »** après une modification.
- L'index des citations bibliques n'affiche plus « Livres canoniques »
  lorsque tous les livres cités sont canoniques : le titre est implicite.
  Un index uniquement non canonique garde « Livres non canoniques » pour le
  signaler ; les deux titres n'apparaissent qu'en présence des deux corpus.
- Vérification : 801 tests réussis, 0 échec, 214 ignorés ; batterie applicative
  182/182 ; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo build --release` et `git diff --check` verts.

## 0.6.1 — les balises bibliques de bout en bout (2026-11-18)

La 0.6.0 a livré la palette et les formes canoniques. Cette version corrige
ce qui restait sur le chemin des citations bibliques et de la saisie.

- L'index des citations reconnaît les balises canoniques `{{b …}}` produites
  par l'application, en plus des anciennes écritures.
- L'index peut séparer les livres canoniques et non canoniques selon le canon
  choisi (protestant, catholique, orthodoxe, téwahedo ou tout le corpus).
- Les titres, sous-titres et figures sont linkables par `[[référence|texte]]` ;
  taper `[[` ouvre la recherche, et les liens sans cible sont signalés dans le
  panneau gauche.
- Un document `.typ` peut être importé et traduit en `.tw` : titres, emphases,
  listes et commentaires passent ; le reste est conservé en bloc Typst brut.
- `{{d aujourd'hui}}` et `{{d maintenant}}` composent la date et l'heure de la
  composition ; `SOURCE_DATE_EPOCH` permet une sortie reproductible.
- Des versions bibliques locales peuvent être ajoutées depuis un fichier JSON,
  avec le nombre de livres et les textes apocryphes propres à chaque édition.
- La forme canonique refuse les liens imbriqués, et la propriété de round-trip
  couvre désormais les liens internes.
- Cliquer une référence ou la survoler fonctionne aussi dans une balise
  canonique : la bulle de verset et la fiche d'édition retrouvent le livre.
- La palette ne découpe plus une chaîne au milieu d'un caractère accentué
  lorsque le curseur revient sur une frontière ; un `É` ne fait plus tomber
  l'application.
- La batterie applicative retrouve son binaire de test, quel que soit le
  répertoire de sortie de Cargo ; les 167 épreuves passent.
- Les épreuves de couleur, de glossaire, de graphique et d'index suivent la
  syntaxe courte désormais produite par l'application.

**Vérification** : 717 tests réussis, 0 échec, 193 ignorés ; batterie
applicative 167/167 ; `cargo fmt --check`, `cargo build --release`,
`cargo clippy --all-targets -- -D warnings`, `git diff --check` et
`cargo deny check` verts.

## 0.6.0 — écrire sans quitter le document (2026-11-18)

### Une seule porte pour les gestes

- **La palette de commandes** réunit les actions, les éléments à insérer et
  les extraits de texte. `Ctrl+K` ou `/` ouvre les commandes, `,,` ouvre les
  balises et les modèles de passage.
- Les commandes et les balises d'un module désactivé disparaissent de la
  palette : l'interface ne propose que ce que le document peut réellement
  utiliser.
- Les formulaires d'insertion montrent désormais des choix utiles : couleurs
  rapides et roue chromatique, formes de date courantes, termes du glossaire
  déjà présents dans le projet.
- Les balises courantes ont une écriture courte et cohérente : `{{c …|…}}`,
  `{{d …}}`, `{{g …}}`, `{{b …}}`, `{{m …}}`, `{{e …|…}}`, `{{gr …}}`,
  `{{t}}`, `{{s}}`, `{{bib}}`, `{{gs}}`, `{{idx}}`. Les anciennes écritures
  restent lues.

### Citations et bibliographie

- **`@` ouvre le composeur de citation** dès la frappe ; il reste possible de
  passer par le bouton de la barre ou par la commande. Une adresse
  électronique ne déclenche rien.
- Le composeur permet de choisir une référence, une page, puis d'en ajouter
  d'autres dans la même note. Une citation déjà écrite se rouvre au curseur.
- Les références viennent de la bibliographie locale, d'un fichier `.bib` ou
  d'une collection Zotero en lecture seule ; les styles de citation Zotero
  sont reconnus.
- Le panneau de bibliographie permet, pour une bibliothèque locale, de
  **tout supprimer** d'un coup — copie complète à la corbeille, commentaires
  du `.bib` conservés —, de **changer de collection Zotero** sans rupture
  préalable et de **rompre le lien**. Les boutons d'édition et de suppression
  d'une entrée restent affichés au lieu de disparaître au survol.
- Le curseur revient dans le document après une insertion, sans perdre le
  fil de l'écriture.

### Chapitres et sommaire

- Une tête de niveau 1 (`# Titre`) ouvre un chapitre ; le panneau gauche les
  liste sous **Chapitres**, avec **Document entier** en tête.
- Choisir un chapitre ne réécrit jamais `document.tw` : le rendu ne montre
  que ses pages, et le **Sommaire** se réduit à ses têtes. Le retour au
  document entier se fait par la même ligne.
- Les pages d'un chapitre sont celles du PDF réel, lues dans le sommaire
  composé ; le numéro de page reste celui du document, pas un numéro local
  qui changerait selon la vue.

### La Bible, jusqu'aux livres hors canon

- Le panneau d'insertion utilise une **liste de livres cherchable** : on
  tape un nom, la liste se filtre, et un livre inexistant ne peut pas être
  choisi.
- Les champs livre, chapitre et versets sont pris en compte au moment
  d'insérer, même sans cliquer sur **Ajouter**.
- Les abréviations de la **TOB** sont proposées, et les deutérocanoniques
  comme les apocryphes — Tobie, Judith, Sagesse, Siracide, Baruch,
  Maccabées, Esdras additionnels, Hénoch, Jubilés, Testaments des
  patriarches, apocalypses — sont citables et indexables.
- L'index biblique donne les pages réelles du document ; les statistiques
  montrent les livres cités et permettent de rejoindre une référence d'un
  clic. Une bulle de verset suit la souris et s'ouvre au-dessus ou en dessous
  selon la place disponible.

### Lire le document final

- Le lecteur de PDF intégré permet de sélectionner et copier le texte
  composé, de chercher une expression, de suivre les liens et de parcourir
  les pages.
- Le numéro de page, le surlignage de la recherche et le défilement ont été
  stabilisés ; le rendu ne bloque plus l'écriture.
- Les documents longs se chargent et se composent plus calmement : le texte
  reste fluide pendant la frappe, les pages se mettent à jour sans lancer de
  travail inutile.

### Un espace de travail plus sûr

- Les copies précédentes, les sauvegardes automatiques et la récupération
  après une écriture interrompue restent disponibles.
- Le verrou de projet empêche deux fenêtres d'écrire le même document.
- Les projets s'exportent en archive portable ou en fichier `.tw` unique.
- Les sauvegardes, la corbeille et l'ouverture du dossier de données sont
  accessibles depuis l'application.

### Mise en page et export

- Les normes du document règlent papier, marges, en-tête et pied de page,
  typographie, titres, couleurs, décor, numérotation et modules.
- Les encadrés, les graphiques, les tableaux, les dates et les métadonnées
  ont leurs formulaires d'insertion ; les métadonnées du projet s'appellent
  comme des balises (`{{m auteur}}`).
- Les exports PDF, Markdown et DOCX restent disponibles, ainsi que le projet
  portable.

### Vérification

- **702 tests réussis / 0 échec / 190 ignorés**, batterie applicative verte,
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo build --release` et `cargo deny check` verts.


## 0.5.0 — les plugins à leur place, les références au clair (2026-11-11)

### Le panneau de travail

- Le panneau gauche s'ouvre sur un bloc **Repères** : nom du document,
  volume, références bibliques, notices et termes du glossaire. Le
  sommaire, les chapitres et les autres sections viennent ensuite.
- Les titres de section sont unifiés — chevron, repli, séparateur — et
  Sommaire, Chapitres, Projets, Bibliographie, Glossaire et Statistiques
  se replient indépendamment.
- Les guides du sommaire ne coupent plus le trait entre deux enfants d'un
  même parent.

### Les références bibliques

- Une balise peut porter une **liste de versets** : `{{Jn 3.16, 17 ; Rm 8.1}}`
  mêle versets du même chapitre et livres différents.
- La forme d'affichage se choisit par balise avec `|long`, `|court`, `|osis`
  ou `|sanspoint` ; les formes relatives (`{{Jn 3.16 ; 17 ; 4.1}}`) sont
  lues. Le standard est le nom abrégé et le point (`Jn 3.16`).
- Un même livre n'est nommé qu'une fois, ses occurrences sont séparées par
  une virgule ; l'onglet **Biblique** des plugins propose l'ordre canonique.
  Les chapitres entiers s'écrivent `{{Gn 1-11}}` et se rendent
  « Genèse 1 à 11 ».
- Le texte libre entre guillemets reste dans la citation
  (`{{Jn 3.16 ; "cf. aussi" ; Gn 1.1}}`), et le panneau d'insertion offre un
  champ pour l'ajouter.
- Le **gabarit de citation** se règle dans l'onglet Biblique : jetons
  `{livre}`, `{chapitres}`, `{versets}`, `{nombres}`, `{chapitre}`,
  `{chapitre_fin}`, `{verset}`, `{verset_fin}`, `{reference}`.
- L'**index des citations** gagne ses réglages : pages réelles demandées à
  Typst, regroupement par livre, forme des livres propre à l'index, nombre
  de colonnes (1 à 4) et suffixe `n` pour les citations en note.
- Le bouton **Référence biblique** ouvre un panneau de construction : livre,
  chapitre, versets (`16`, `16-18`, `16, 17`), plusieurs références, puis
  **une seule** balise qui les porte toutes.
- Le panneau propose la forme longue, abrégée avec ou sans points, OSIS ou
  l'héritage des normes ; ce choix ponctuel ne modifie pas le projet.
- **La référence cliquée** sépare les citations dans le panneau gauche,
  montre le livre et le chapitre de chacune, et son crayon ouvre une fenêtre
  flottante pour modifier une référence sans quitter le document.
- Les **statistiques bibliques** listent les références réellement citées,
  avec leur nombre d'occurrences — la mémoire des versets du document, pas
  un classement des livres. Elles se cherchent, se trient dans l'ordre du
  document ou dans l'ordre canonique, et un clic place le curseur sur la
  citation.
- L'**index des citations** s'insère par l'icône de la barre supérieure ou
  la balise `{{index}}`, et se recalcule au rendu, comme la table des
  matières.
- Le survol d'un verset n'exige plus Ctrl : après un court repos, une boîte
  de largeur maximale 420 px s'affiche au-dessus du curseur, centrée, et
  bascule dessous si le haut manque de place. Le verset montré est celui qui
  se trouve sous la souris, y compris dans une liste.

### Les plugins et la bibliographie

- Les **normes** gardent l'activation et la désactivation des add-ons.
  Les paramètres des plugins ont leur propre panneau, ouvert par le bouton
  correspondant de la barre d'outils : Bibliographie, Biblique, Glossaire
  et Graphiques. Il reprend la fenêtre des normes — mêmes dimensions, même
  rail, même pied — et se distingue par son contenu seul.
- L'icône des **normes** n'est plus marquée active quand ce panneau est
  ouvert ; chaque icône ne l'est que pour son propre panneau.
- Le style bibliographique, l'import `.csl`, les styles Zotero et la forme
  des références bibliques sont dans ce panneau, plus dans les normes.
- La forme des livres, les chiffres (arabes ou romains) et les séparateurs
  se règlent pour le projet. Une traduction ne s'affiche dans une citation
  que si celle-ci la nomme (`{{Jn 3.16|lsg}}`, `{{Jn 3.16|darby}}`).
- Une bibliographie ne s'ajoute plus dans le dos de l'auteur : ni le modèle
  de projet ni le branchement automatique ne posent `#bibliography` sans
  directive explicite. Le `.bib` seul n'est pas une instruction.

### L'écriture et l'interface

- Le sélecteur de couleur rapide prévisualise ; seule l'application écrit
  la balise, une fois, puis ferme.
- Les couleurs explicites (`#text(fill: …)`) ne sont plus écrasées par le
  gabarit.
- Toute fenêtre au premier plan bloque les clics derrière elle, et le
  glossaire n'ouvre qu'une fenêtre à la fois — liste ou fiche.
- La complétion ne s'ouvre plus sur `{` : la référence biblique s'ouvre
  par un geste explicite, et le curseur se pose entre les accolades.
- Les déclencheurs de guillemets se personnalisent (ouvrants et fermants) ;
  la paire produite reste celle du niveau d'imbrication.
- Le panneau des plugins et les fenêtres se conduisent au clavier, avec le
  focus pris à l'ouverture.

### Qualité

- L'application est découpée en modules (`base`, `design`, `services`,
  `plugins`, routeurs d'application) ; le dispatch central perd les
  familles de messages au profit de routeurs dédiés.
- Les états de plugins vivent par vue et suivent les changements d'onglet.
- Les portes de vérification sont dans la CI : `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo deny check` et la
  batterie applicative.
- **598 tests réussis / 160 ignorés**, batterie **140/140**, clippy zéro.

## 0.4.0 — une base de travail complète (2026-11-07)

Version de transition : l'assistance à la frappe passe par **tinymist**,
facultatif, l'application est découpée en modules (`rendu`,
`rasterisation`, `saisie`, `fichiers`, `projet_dialogue`), les diagnostics
du compilateur entrent dans le produit et la découverte des polices passe
par `fontdb`. Cette version a été publiée sans notes détaillées ; les
fonctionnalités R1–R8 restent décrites dans la section 0.3.0-alpha.3
ci-dessous.

## 0.3.0-alpha.3 — le coffre-fort des documents et le quotidien

R1 à R8 de la feuille de route, les
rapports 7 à 10, et les corrections qui ont suivi. Les limites
connues sont nommées dans le [README](README.md) — les essais
Windows et macOS restent à faire nativement.

### R1 — le coffre-fort des documents

R1 de la feuille de route : sauvegarde,
verrou et écritures sûres.

- Un verrou natif protège les instances récentes contre les acquisitions
  simultanées ; le protocole a été testé sous Linux, pas encore sur Windows
  ou macOS. Ne travaillez pas simultanément avec une ancienne version, qui
  ne respecte pas ce verrou. Rust 1.92 est requis pour construire le projet.
- Les migrations au démarrage ne modifient plus un projet dont le verrou
  n'a pas été obtenu ; la migration des anciens fichiers à plat s'abstient
  lorsqu'un projet existe déjà.
- Les écritures atomiques utilisent des temporaires distincts et conservent
  les permissions du fichier remplacé. Ces mesures ne remplacent pas une
  sauvegarde : le verrouillage, la récupération et la fermeture sûre sont
  encore en chantier.
- Avant chaque remplacement, la version précédente du document est conservée
  dans `sauvegardes/`, à côté du projet : vingt copies par document,
  espacées d'une minute. Un fichier modifié hors de l'application n'est
  jamais écrasé en silence — une question s'ouvre, la version du disque est
  conservée quoi qu'il arrive. La procédure de restauration est au guide.
- Un export — PDF, mise en page, copie — refuse d'écrire dans un projet de
  l'application que la fenêtre ne détient pas, et les retours d'une
  compilation périmée ne remplacent plus l'aperçu courant.
- Une écriture interrompue par une panne ne se perd plus : le temporaire
  abandonné devient, à la réouverture du document, une copie datée dans
  `sauvegardes/`. Une copie de sécurité ou une écriture impossible ne
  remplace jamais le document, et le renommage des fichiers est synchronisé
  pour survivre à une coupure de courant.
- La création d'un projet ne paraît qu'une fois prête : le squelette se
  prépare dans un temporaire caché, puis le projet est publié d'un seul
  renommage — jamais un demi-projet dans la liste.
- Lors d'une fermeture normale, les documents modifiés sont enregistrés
  avant de fermer la fenêtre ; en cas d'échec, la fermeture est annulée et
  le tampon conservé. Les épreuves couvrent les messages et les fichiers ;
  les gestes réels restent à vérifier.
- Importer une image extérieure n'écrase plus une image du même nom : la
  nouvelle copie reçoit un suffixe numérique.
- Plusieurs écritures du projet et la frappe sont refusées lorsqu'aucun
  verrou n'a été obtenu. La couverture des autres opérations et le
  verrouillage multiplateforme restent à compléter.
- Avant le déplacement ou la suppression d'un projet, les vues modifiées
  sont sauvegardées ; la suppression est annulée si cette sauvegarde échoue.
  Les vues et la session suivent le nouveau dossier lors d'un renommage.
- Les épreuves qui lisaient des projets personnels utilisent des fixtures
  synthétiques.


### Le glossaire (glossarium)

- Le glossaire se gère du **panneau gauche** : un terme porte sa clé, ses
  formes courte et longue, ses pluriels, sa définition et son groupe.
  Un clic insère sa référence au curseur ; le crayon corrige, la
  corbeille supprime ; la loupe cherche par clé, formes ou définition.
- Les balises `{glo:clé}`, `{glo:clé pluriel}`, `{glo:clé majuscule}` et
  `{glo:clé majuscule pluriel}` appellent le terme ; la ligne
  `{glossaire}` pose la liste finale où l'auteur la veut —
  `{glossaire: tout}` y montre aussi les termes jamais cités.
- Le rendu est celui du paquet **glossarium 0.5.10**, vendé dans le
  dépôt et installé localement à la compilation — **jamais de réseau** :
  première occurrence développée, reprises serrées, liste avec renvoi
  aux pages. Sans liste, les termes se composent sans lien ; un terme
  inconnu se neutralise en `[clé]` et se signale — le document compile
  toujours.
- Les termes vivent dans `glossaire.toml`, à la racine du projet, dans
  l'ordre de l'auteur ; ils voyagent avec le projet (archive portable et
  `.tw` unique — sa clôture passe à la version 2, qu'une ancienne
  application refuse proprement au lieu de tout lire de travers).

### R2 à R4 — imports sans perte, références fidèles, styles bibliographiques


R2 de la feuille de route :

- L'import d'un fragment Typst ou d'un fichier `.tw` travaille sur le
  **tampon ouvert** : la conversion est prévisualisée — ce qui s'est
  converti, ce qui reste brut — avant d'y entrer, d'un seul geste
  annulable ; le fichier d'origine n'est ni modifié ni supprimé, et rien
  ne s'écrit qu'à la sauvegarde. Les frappes non sauvegardées survivent
  à tout import.
- Un tableau irrégulier ne perd plus aucune cellule : la ligne la plus
  large élargit le tableau au lieu de la troncature silencieuse qui
  l'attendait. Les cellules unicode se relisent telles quelles.
- Supprimer une notice est une vraie corbeille : l'entière part à la
  corbeille avant toute suppression — la bibliographie ne bouge pas si la
  copie échoue — et se restaure d'un geste, sans jamais dédoubler une clé.
- Le renommage de symbole nomme les fichiers qu'il a touchés, et un
  fichier illisible l'arrête au lieu de disparaître du résultat.
- Une citation inconnue n'est plus un silence : la neutralisation en
  `[clé]` se dit avec sa clé, sa ligne et son fichier, dans les
  avertissements de compilation — panneau de l'aperçu et barre d'état.
- Une image homonyme ne s'écrase jamais : la copie prend un nom libre et
  le document la cite. Un import dont le fichier d'origine a changé
  depuis l'aperçu se refuse au lieu de coller un texte périmé.
- Les clés non ASCII sont des clés à part entière, et une clé non ASCII
  manquante se neutralise au lieu de casser la compilation. Une adresse
  électronique n'est jamais une clé : elle s'échappe et se relit intacte.
- Un aperçu affiché alors que la compilation actuelle a échoué se dit
  « ancien » : les pages sous les erreurs ne sont plus celles du texte.
- Le composeur de citation trouve l'auteur, le titre et l'année, pas
  seulement la clé — la recherche de la bibliothèque, partout.
- Les clés d'une bibliographie portée par un chapitre sont vivantes, et
  une clé commune à deux bibliographies produit un diagnostic de
  collision nommant les deux sources. Le texte extrait du PDF porte la
  source citée : vérifié sur le PDF réel, pas seulement au compilateur.
- Le contrat de source bibliographique partagé est écrit :
  `agent/Pré-Beta/contrat-source-bibliographique.md` — la source unique,
  ses consommateurs, et sept invariants vérifiés par leurs épreuves.
- Le style des citations et de la bibliographie se choisit dans les
  normes du projet, parmi les styles réellement connus du moteur
  embarqué, avec ses règles de priorité explicites : la directive du
  document n'est jamais écrasée en silence, et un style inconnu n'est
  jamais appliqué.
- Un style `.csl` local s'importe et suit le projet : contrôlé par le
  même analyseur que le moteur, copié sous `styles/` sans écraser,
  l'original intact. Un style dépendant est refusé avec le nom de son
  parent, jamais accepté en silence.
- La fenêtre des normes prévisualise citation et bibliographie par le
  moteur même, et dit quel style est regardé.
- Un style dépendant se résout par son parent — cherché dans le projet ou
  dans les styles Zotero, copié sans rien modifier — avec son écart de
  locale dit ; parent absent, le téléchargement n'est jamais proposé
  qu'avec consentement, et le dernier rendu valide reste en place.
- Les styles installés dans Zotero se choisissent depuis la fenêtre des
  normes : copiés dans le projet, jamais modifiés, et un style disparu
  se dit au lieu d'être remplacé en silence.
- L'export Zotero écrit clés, bibliothèque et liaison en une transaction —
  tout, ou rien, avec l'état d'avant remis au moindre échec — et une
  réponse arrivée après un changement de projet n'écrit dans aucun autre.
- Un `.bib` exporté par Zotero s'importe sans plugin : rien n'est écrasé,
  les conflits se disent, et un `@` dans un commentaire n'est plus une
  fausse notice.
- Le moteur Typst est épinglé à sa version exacte : un glissement de
  version ne peut plus changer un rendu sans que les tests ne le disent.

### R5 — modèles de documents, typographie, notes et renvois


- Deux modèles de documents en un geste : devoir court et travail long,
  naissant complets — page de titre, sommaire, bibliographie — sans
  institution imposée, avec un en-tête courant résolu par le moteur. Un
  projet existant garde son rendu, sans choix explicite.
- La validation des normes suit les réglages par niveau comme les
  globaux — plus aucune valeur de titre avalée en silence — et
  l'application des titres comme des champs dynamiques est prouvée dans
  le PDF extrait.
- L'aperçu des titres est un vrai rendu du moteur avec les normes du
  dialogue, sans rien écrire au projet avant l'application.
- Les références croisées ne sont plus prises pour des citations
  manquantes : `@fig:une` et les étiquettes du document traversent le
  correcteur intacts, et une cible manquante se diagnostique par son nom.
- L'audit de cetz est fait : le moteur ne résout pas les paquets Typst,
  les graphiques ne sont pas présentés comme fiables — la garde le dit.
- Note, figure, étiquette et renvoi s'insèrent d'un geste : le gabarit
  entre au curseur, le curseur se pose là où l'on doit écrire — et le
  dialecte est celui du tampon (`^[…]` en .tw, Typst dans les chapitres).
- Devoir court et travail long vérifiés de bout en bout : titres,
  sommaire, sauts, notes, légendes, tableaux, renvois, bibliographie et
  export — lus dans le PDF extrait.

### R6 à R8 — édition quotidienne, exports, paquet Linux

- Bug-report 7, premier lot : les siècles deviennent de vraies petites
  capitales à l'exposant (le code mort dans le rendu vient de là),
  l'étiquette s'écrit dans les deux dialectes, les boutons de format
  écrivent même sur un bloc vide (ils se taisaient sur les projets neufs),
  le bouton du document d'écriture répond toujours, l'aperçu se
  rafraîchit à la bascule.
- La couleur du texte se choisit sur une roue (teinte, saturation,
  valeur), centrée dans la fenêtre, avec aperçu hex et ouverture sur la
  dernière couleur. Les quick parts de l'en-tête et du pied fonctionnent —
  `{projet}`, `{document}`, `{date}`, `{auteur}`, `{titre}` — les entrées
  du moteur n'étaient jamais alimentées et les marqueurs étaient
  défigurés par leur propre échappement ; les images s'y posent aussi, et
  chaque champ a son sélecteur de quick parts.


- La recherche/remplacement est exacte dans ses deux modes : le littéral
  ne lit ni métacaractères ni `$`-groupes, le régulier les garde.
- La recherche montre ce que chaque occurrence devient avant de remplacer,
  et l'undo remet tout d'un geste — remplacements comme notes supprimées.
- Les raccourcis exigent le Contrôleur (verrouillé par tests), et un
  aperçu de normes ou de thème annulé n'enregistre jamais rien.
- Un projet s'exporte en une archive portable (avec l'inventaire de ce
  qui manque) et se réimporte : jamais d'écrasement sans choix, ce qui est
  remplacé reste dans la corbeille, et un import interrompu remet tout en
  place. L'aide (F1) devient le centre d'aide, diagnostic et récupération. L'expérience DOCX/Markdown est
  mesurée et documentée : titres, tableaux, citations et images y
  survivent, les notes deviennent du texte. Les deux exports en découlent :
  Markdown fidèle (canonique, relu au même document) et DOCX par
  LibreOffice, avec sa limite dite — et son remède dit quand LibreOffice
  - Les performances d'un long document et d'une bibliothèque de
    1 000 notices sont mesurées et publiées, sans goulet caché dans la
    chaîne d'écriture. Le dossier des données et l'ouverture des dossiers
    sont portables (Linux, macOS, Windows) sans rien déplacer sous Unix.
  L'aide (F1) ouvre sur les premiers pas et sur le chemin des données,
  ouvrable d'un geste. Un paquet Linux s'installe, se met à jour et se
  désinstalle d'un geste, chaque fichier vérifié par son empreinte.
- Le changement d'onglets ne perd jamais le tampon, une bibliographie de
  300 notices se charge sans troncature, et le mode focus rend la
  disposition d'avant — éprouvés. Choisir une notice ouvre sa fiche sans
  rien écrire d'autre, la note du composeur s'écrit dans le texte, et un
  saut de diagnostic ne touche ni la saisie ni l'annulation.

### Bug-report 8 — le panneau des normes en entier (2026-11-04)

- Les neuf chantiers du rapport 8 sont traités : polices système et
  suggestions, quick parts vivantes partout, couleurs unifiées
  pastille→roue, grille des titres (couleur, italique, souligné),
  images d'en-tête avec sélecteur natif et hauteur en %, guillemets à
  préréglages avec aperçu à trois niveaux, formats de dates documentés
  et noms localisés fr/en/de/it, aperçu CSL fidèle, composants
  harmonisés et persistance éprouvée. Deux courses parallèles du cœur
  (corbeille, atelier d'import) trouvées en chemin et réglées à la
  racine.

### Bug-report 9 — le rendu PDF, les normes en main, Zotero (2026-11-05)

- Le rendu vient du **PDF** : la fenêtre ne rasterise que ce qu'elle
  montre, par région ; le zoom et la molette sont ancrés au curseur,
  avec retour immédiat — la molette dirigée n'intercepte que Ctrl.
- Le panneau des normes charge la norme du document (« Normes
  personnalisées » hors correspondance), dit ce qu'on modifie, et
  choisit d'écraser ou d'enregistrer une copie nommée ; les couleurs de
  style (gras, italique, liens) se rendent réellement au moteur.
- Le lien **Zotero** branche la bibliographie : l'appel `#bibliographie`
  cite exactement les `.bib` présents ; les sections `{bibliographie}`
  et `[[TOC]]` s'insèrent d'un geste ; la numérotation est libre partout
  (tout schéma passe tel quel).

### Bug-report 10 — la barre, le glossaire, les listes, la synchro (2026-11-05)

- La barre supérieure : BIB/GLO en icônes, styles de titre en icône et
  menu, séparateur fin entre les fenêtres ; le sélecteur de couleur
  distingue le choix (« Enregistrer ») de l'application (« Appliquer »).
- Le glossaire s'ouvre en **fenêtre dédiée** : consulter, chercher,
  insérer, ajouter, modifier, supprimer.
- Structure documentaire : saut de page avant la bibliographie selon
  les normes, bibliographie en titre de niveau 1, balise `[[TOC]]` et
  bouton ; le sommaire gauche est continu, la bibliographie se replie
  par bloc avec recherche, Projets et bibliothèque gèrent d'un geste.
- Des **profils de métadonnées** portent leurs valeurs, les balises
  `{clé}` les lisent, les clés système sont réservées.
- **Listes multiniveaux** : deux espaces de retrait par niveau, `Tab`
  et `Maj+Tab` changent le niveau, `Ctrl+Maj+Haut/Bas` déplace
  l'élément, `Entrée` crée l'élément suivant.
- Le rendu : les bibliographies dupliquées ne cassent plus la
  compilation (la bibliothèque locale gagne sur l'export Zotero), et
  la synchro édition / rendu se déclenche à la bascule de mode, à
  1,5 s d'arrêt de frappe et à `Entrée`.

### Corrections après le rapport 10 (2026-11-06)

- La bibliographie du panneau gauche se replie **en un bloc** ; la
  recherche montre les entrées qui matchent même repliées.
- Chaque fenêtre de rendu d'un même fichier partage **le même PDF** —
  un registre par empreinte, jamais une copie par fenêtre.

**Comptes** : 553 tests ordinaires, 126 épreuves d'espace, 29 tests du
crate vendoré, clippy 0 warning, `cargo deny check` vert.

## 0.3.0-alpha.2 — rapports 5 et 6 soldés, interface fiabilisée

Les rapports de bugs 5 (23 demandes) et 6 (8 demandes) sont
intégralement traités. Corrections visuelles et fonctionnelles.

### Navigation et onglets

- **Shift/Ctrl+clic** sur un projet = nouvel onglet (sans remplacer l'actif) ;
- **clic sur onglet** ne split plus la fenêtre (vérification de distance) ;
- **Ctrl+Maj+Tab** bascule entre panneaux ;
- dépôt d'onglet hors panneau = nouveau panneau.

### Interface

- **top bar** : trois groupes équilibrés (général / édition / thème) ;
- **bouton de fermeture** dans la barre, plus dans le panneau ;
- **panneau latéral** : sommaire en haut, noms de document supprimés ;
- **alignements** : textes centrés verticalement dans leurs boutons ;
- **menu des titres** : `pick_list` au lieu de `tooltip` (restait ouvert) ;
- **police** : sélecteur retiré de la barre (normes uniquement) ;
- **aperçu** : boutons supprimés (gérés depuis l'espace).

### Éditeur de thème

- **picker RVB intégré** : sliders R/G/B, aperçu, Valider/Annuler ;
- **zenity supprimé** : plus de fenêtre native séparée ;
- **pastille cliquable** : ouvre le picker directement ;
- **fond synchronisé** : l'éditeur suit toujours le fond de l'application.

### Normes

- **config par niveau de titre** : couleur, graisse, espacement, casse ;
- **flèches** uniquement sur les champs numériques ;
- **modèles** en haut de la fenêtre.

### Éditeur

- **suppression de glyphe** : `has_selection()` au lieu de `anchor.is_some()` ;
- **overlay de note** : fermeture au clic extérieur ;
- **rendu des dernières lignes** : étendue de rendu corrigée ;
- **surlignage** des occurrences de recherche.

### Recherche

- **mode littéral / regex** : bouton `Aa` / `.*` ;
- **surlignage** dans l'éditeur.

### Qualité

- **clippy 0** projet et vendor ;
- **nettoyage** : imports explicites, `enumerate()`, `#[allow]` documentés ;
- **comptage bibliographique** fiabilisé.

## 0.3.0-alpha — suite : le rapport 4 soldé, la qualité au sommet

Le rapport de bugs 4 (63 demandes d'interface et de qualité) est
intégralement traité. Nettoyage complet du code et du vendor.

### Le langage

- **saut de page** : trois lignes vides consécutives → `#pagebreak()` ;
- **souligné et barré** : `__texte__` et `~~texte~~` ;
- **commentaires** : `// texte` — conservé dans le `.tw`, ignoré au rendu ;
- **autoformatage des tableaux** : les lignes du corps sont normalisées
  au nombre de colonnes de l'en-tête.

### La recherche

- **mode littéral / regex** : bouton `Aa` / `.*` — le texte est pris au
  pied de la lettre ou interprété en expression régulière ;
- **surlignage des occurrences** dans l'éditeur à chaque frappe.

### Les normes

- **Quick Parts** : `{document}`, `{date}`, `{auteur}`, `{projet}` dans
  l'en-tête et le pied de page — dynamiques, jamais figés ;
- **casse par niveau de titre** : chaque niveau (1-6) a sa propre casse ;
- **aperçu de l'imbrication des guillemets** en temps réel.

### L'interface

- **sommaire en arborescence** : lignes de connexion `│ ├── └──` ;
- **barre supérieure** : sélecteur de police, pastille de couleur
  courante, séparateurs par famille de commandes, icônes alignées ;
- **éditeur de thème** : sections (Fond / Texte / États), color picker
  natif (zenity), infobulle sur la pastille ;
- **panneau latéral** : bouton de réduction dans le panneau, année de
  publication dans la bibliothèque, filtre par année ;
- **espace de travail** : `Ctrl+Maj+Tab` bascule de panneau, dépôt
  d'onglet hors panneau = nouveau panneau ;
- **mode focus** : le rendu est centré.

### La qualité

- **clippy 0** dans le projet et le vendor ;
- **nettoyage du vendor** : imports explicites, `enumerate()` au lieu de
  compteurs manuels, `#[allow]` documentés ;
- **comptage bibliographique fiabilisé** : une entrée tronquée n'emporte
  plus les suivantes ;
- `PATCHES.md` : 17 points documentés.

## 0.3.0-alpha — l'espace de travail, la bibliothèque, et les trois rapports soldés

La version où l'éditeur est devenu un **espace de travail** — panneaux
divisibles, onglets par panneau, disposition restaurée —, où la
bibliothèque de projets prend toute la fenêtre, où les graphiques
entrent dans la langue, et où les **trois rapports de bugs sont
soldés** (18 + 14 + 17 points).

### L'espace de travail

- **Panneaux divisibles** : l'éditeur se scinde en autant de panneaux
  qu'on veut (bord de 8 px), chacun avec ses onglets ; glisser un onglet
  le déplace, le détache ou le divise ;
- **onglets par panneau**, nommés par projet (`document.tw` → le nom du
  projet) ; un point de mode coloré dit l'état (bleu édition, vert
  rendu, orange Typst) ; le × apparaît au survol ;
- **même document, plusieurs vues** : texte partagé, vues indépendantes
  (édition / rendu / Typst) — chacune scrolle et zoome à son gré ;
- **disposition restaurée** : panneaux, onglets, modes et documents se
  retrouvent au lancement ; les vues orphelines ne sont pas sérialisées ;
- **page d'accueil** dans les panneaux vides : Créer, Ouvrir, Bibliothèque ;
- **shift+clic** = nouvel onglet, **clic simple** = remplace l'onglet actif
  (sauvegarde avant fermeture) ;
- **focus mode** maximise la vue active (édition, rendu ou Typst).

### La bibliothèque

- **galerie plein écran** : les projets se voient par couvertures
  cliquables, rangés sur des étagères ;
- **étagères** : classement des projets par drag-and-drop,
  `etageres.json` ; bouton « + Nouveau projet ici » en fin de rangée ;
- les couvertures sont des poignées d'image stables (empreinte de
  pixels), jamais recréées.

### La langue

- **graphiques** : `{graphique type=barre titre="Titre"}` suivi de lignes
  `label | valeur` — trois types (barre, ligne, secteur), rendus par le
  paquet Typst `cetz` ; l'import n'est ajouté que si le document
  contient un graphique ;
- **légende d'image par `|`** : `![alt](chemin | légende)` — la légende
  emporte sur l'alt, numérotée par les normes ;
- **notes de bas de page** : clic sur `^[]` → overlay en bas de l'écran,
  Entrée enregistre, Échap ferme — sans déplier la note ni déranger le
  fil du document.

### L'interface

- **éditeur de thème complet** : ouverture sur le thème actif, application
  immédiate, bouton « Réinitialiser » (recharge depuis le disque) ;
- **Ctrl+1-6** insère un titre au curseur (`#`×n + espace), jamais de
  reformatage de bloc ;
- **molette et flèches** sur les champs numériques des normes (la valeur
  garde son unité : `11pt` → `12pt`, `2cm` → `3cm`) ;
- **en-tête et pied de page** dans les normes : contenu libre,
  numérotation multiple (a, A, 1, p. 1, page 1 sur 12, 1/12…) ;
- **icône titre hover** : menu H1-H6 au survol ;
- **liseré actif** limité à la barre d'onglets ;
- **boutons inutiles supprimés** (renommer symbole, position aperçu) ;
- **clavier de la fiche de référence** : Tab/Entrée/Échap ne sont plus
  avalés par le composeur fermé ;
- **contraste renforcé** : `text_muted` calculé par mélange de
  luminance, tenue dans tous les thèmes ;
- **mode Typst** : texte généré complet (pipeline `assembler`), plus de
  `#include` seul.

### La qualité

- **`deny.toml` configuré** : 13 licences SPDX permissives autorisées,
  11 advisories documentés avec raison, sources strictes (crates.io
  uniquement) — `cargo deny check` passe (exit 0) ;
- **zéro `unsafe`** dans `src/`, 243 `unwrap()` dont 95 % en tests ;
- **fuite mémoire corrigée** : l'aperçu de police fuyait une `String` à
  chaque frappe (`Box::leak` → `intern_family` avec cache) ;
- **12 `Eq`** ajoutés sur les types qui dérivaient `PartialEq` sans
  raison ; 5 `const fn` ; 5 clones redondants supprimés ;
- **zéro allocation** dans la sélection des polices (`&str` au lieu de
  `String`).

**Comptes** : 395 tests projet, 654 tests du crate vendoré, clippy 0,
`cargo deny` ok.

## 0.2.0 — la citation, le langage, et l'interface

La version où le document est devenu une **couche simple** (`document.tw`) entre
les doigts et Typst, où les citations se sont faites **glyphes**, et où
l'interface s'est tue pour laisser parler le texte.

### Le langage intermédiaire

- `document.tw` : titres, citations `@cle[p. 45]`, notes libres `^[…]`, blocs
  bruts pour le reste ; sérialisation canonique, migration des projets `.typ`
  hérités sans rien détruire ;
- la syntaxe, l'AST et les deux traductions sont documentées dans
  `docs/design/langage-intermediaire.md` ; le même document génère toujours le
  même corps Typst.

### Les citations

- les citations deviennent des **glyphes numérotés dynamiques** (¹ ² ³) — appels
  de notes et notes libres mélangés dans un seul compteur, dans l'ordre du
  document ;
- une note est un **caractère entier** : curseur, sélection, suppression ;
- **cliquer sur un glyphe** ouvre le composeur et déplie la note au pied de la
  page ; la frappe n'ouvre jamais l'édition, et `Échap` abandonne sans trace ;
- le composeur se conduit aux flèches (la note, puis les clés) et réécrit chaque
  référence à son rang ;
- correctifs mesurés : le clic tombe où l'on clique malgré les glyphes, la
  frappe après un glyphe reste du texte (elle grossissait la clé), la flèche
  gauche sort d'avant une note, la suppression emporte la note entière et une
  seule annulation reprend tout le geste, le remplacement d'une citation ne
  mange plus le texte d'avant, le glyphe est posé aussitôt l'insertion (plus de
  clé à nu dans le `.tw`).

### Les références

- le panneau latéral liste **auteur, prénom, titre** (abrégés s'il faut), jamais
  les clés ; le crayon ouvre la fenêtre d'édition ;
- la fenêtre réécrit les champs de l'entrée dans `references.bib` — et rien
  d'autre ;
- Zotero : une collection pointée se **lit** ; seules les bibliothèques locales
  s'écrivent.

### L'interface

- **Onze thèmes livrés** — Everforest, Moka, Gruvbox, Pin rosé, Solarized,
  Kanagawa (clairs et sombres) — semés une seule fois, jamais réécrits après,
  avec les thèmes clair et sombre d'origine ; un thème est un petit fichier
  TOML ;
- la **sélection se teinte par la bordure**, jamais par le bloc de couleur ; les
  coins s'arrondissent (rayon 12) ; un seul langage de dessin (jetons de rayon,
  d'espacement et de corps de texte) partout ;
- la fenêtre des normes de mise en page : **infobulles** sur chaque paramètre,
  **listes déroulantes** partout où le choix est fini (langue, numérotation des
  chapitres…) ; toute mesure Typst reste acceptée ;
- le contraste du texte posé sur un fond coloré se **mesure** (luminance), donc
  il tient dans tous les thèmes.

### Robustesse

- gardes fondues depuis le dépôt distant : une clé sans virgule ne garde pas son
  accolade, une étiquette vide ne matche rien, un dossier nommé `references.bib`
  n'est pas une bibliographie ;
- nettoyage : `tower-lsp` et trois fichiers vides retirés ; le chantier des
  inutiles est tenu à chaque release.

**Comptes** : 279 tests projet, 653 tests du crate vendoré, clippy 0.

## 0.1.0-alpha.1 — alpha fermée

La première : écrire en Typst avec le rendu du document final page par page,
l'assistance tinymist (complétion, survol, définition, renommage avec aperçu,
diagnostics), les projets à chapitres, l'export PDF, les thèmes clair/sombre et
le mode focus.
