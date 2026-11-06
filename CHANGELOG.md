# Journal des versions

> Les notes historiques ci-dessous reflètent les annonces de publication ;
> elles ne constituent pas une validation des fonctionnalités. L'audit
> ultérieur et les limites constatées sont documentés dans
> [l'audit avant bêta](agent/Pr%C3%A9-Beta/audit-avant-beta-2026-11.md).

## 0.3.0-alpha.3 — le coffre-fort des documents et le quotidien

R1 à R8 de la [feuille de route](roadmap-vers-beta-2026-11.md), les
rapports 7 à 10, et les corrections qui ont suivi. Les limites
connues sont nommées dans le [README](README.md) — les essais
Windows et macOS restent à faire nativement.

### R1 — le coffre-fort des documents

R1 de la [feuille de route](roadmap-vers-beta-2026-11.md) : sauvegarde,
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


R2 de la [feuille de route](roadmap-vers-beta-2026-11.md) :

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
