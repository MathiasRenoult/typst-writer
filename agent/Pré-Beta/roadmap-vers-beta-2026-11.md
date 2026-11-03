# Feuille de route : de `0.3.0-alpha.2` à la bêta

Proposition du 1er novembre 2026 ; aucune étape ci-dessous n'est déjà livrée du seul fait d'apparaître ici. L'ancienne [feuille de route](feuille-de-route.md) reste conservée comme historique mais ses affirmations « fait » et son gel fonctionnel ne valent pas validation du présent plan. Bases : [audit](audit-avant-beta-2026-11.md) et [étude de marché](etude-marche-et-csl-2026-11.md).

## Décisions de cadrage

- Public : l'auteur et sa thèse, mais aussi des étudiants qui doivent pouvoir installer et utiliser seuls l'application.
- Bêta : produit complet pour son parcours central, encore perfectible et extensible ; ce n'est ni une certification de conformité universelle des styles, ni un clone de Word.
- Plateformes exigées : Linux, Windows et macOS. Application et données utilisables hors ligne ; Zotero facultatif et lu seulement.
- Styles : choix parmi les styles disponibles du moteur Typst embarqué, import d'un `.csl` et choix parmi les styles Zotero installés. Chacun doit régir citations et bibliographie finale ; résolution explicite des parents CSL dépendants, sans accès réseau implicite.
- La question d'un import/export DOCX obligatoire n'a pas été tranchée. On mesure les pertes et on recommande, sans construire une dépendance bêta sur une hypothèse.
- Pas de calendrier imposé. Numéros ci-dessous indicatifs : vérifier les tags disponibles avant chaque publication ; conserver l'ordre des dépendances même si les noms changent.

## Contrat commun à chaque release

Une étape = une release contenant au moins deux fonctionnalités ou chantiers réellement livrés, testés et documentés. On produit un paquet vérifié et des notes sincères, puis on ne passe à la suivante qu'après les critères listés. Une régression de sûreté interrompt l'ordre prévu. Les preuves doivent porter sur le comportement final et les fichiers réellement produits, pas sur l'existence d'un bouton.

Pour chaque étape :
1. Définir les invariants, tests reproductibles sur données synthétiques et migration rétrocompatible ; identifier formats et API réels avant codage.
2. Tester unités/intégrations, compilation Typst et contenu/structure du PDF, données sauvegardées et réouvertes ; tester aussi les erreurs, annulation et undo si applicables.
3. Lancer build, tests, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` et tests vendor avec les features effectivement utilisées. L'état de départ n'est pas formaté ; régulariser dans un lot séparé plutôt que masquer l'échec.
4. Tests automatisés avec HOME/profils jetables et paquets réseau explicitement contrôlés ; ne jamais utiliser les vrais projets ou la vraie session. Tests manuels de l'application en fonctionnement sur les OS concernés, en notant les résultats et ce qui n'a pas été testé.
5. Mise à jour ciblée du guide et du changelog, contrôle des licences, CI et archive ; le compte de tests n'est pas une preuve de fonctionnalité. Les prérequis extérieurs sont optionnels ou installés/diagnostiqués avec un geste clair.

## R1 — `0.3.0-alpha.3` : coffre-fort des documents

**Livrables**
- Sauvegarde et fermeture : intercepter la fermeture, attendre la sauvegarde de tous les documents modifiés, signaler et conserver les tampons en cas d'échec ; sauvegardes/versionnement récupérable, procédure de restauration visible.
- Verrou et lecture seule : prise/reprise robuste et atomique sur chaque OS ; un seul service d'autorisation pour toutes les mutations (documents, références, images, normes, migrations). Aucune écriture de projet avant verrou.
- Écritures sûres : temporaires uniques, remplacement correct selon OS, stratégie explicite de durabilité et nettoyage ; versions antérieures et conflits externes détectés.

**Acceptation** : frappe puis fermeture immédiate ; permissions refusées, disque plein simulé, interruption à chaque étape, deux instances et deux projets, réouverture/retour arrière ; à chaque cas, aucune version validée ne disparaît et la récupération est expliquée. Tester aussi l'ouverture d'un vieux projet sous verrou refusé.

## R2 — `0.3.0-alpha.4` : import sans perte et source fiable

**Livrables**
- Import Typst et fichier `.tw` : travailler sur le tampon ouvert, prévisualiser les transformations et préserver l'original ; synchronisation avec undo/redo et protection des changements extérieurs.
- Tableaux et images : ne jamais tronquer silencieusement une cellule ; insertion d'image sans écraser une homonyme, avec noms/choix et copie réversible.
- Suppression de notices et modifications multi-fichiers : corbeille réelle, restauration et vérification des versions ; renommage LSP avec aperçu, périmètre et résultat détaillé.

**Acceptation** : import avec modifications non sauvegardées, tableaux irréguliers et Unicode, images homonymes, annulations, erreurs à mi-opération, conflits entre changements internes/externes ; tests de round-trip et données avant/après identiques sauf transformation explicitement acceptée.

## R3 — `0.3.0-alpha.5` : références locales correctes

**Livrables**
- Un contrat de source bibliographique partagé entre bibliothèque visible, complétion, citations, aperçu et PDF. Choix/import de `.bib` local, détection des erreurs et collisions, édition de notices sans altérer les champs non modifiés.
- Compilateur fidèle : ne plus neutraliser silencieusement une citation ou un chemin bibliographique ; diagnostic accessible avec fichier/ligne et action, dernier aperçu valide clairement indiqué comme ancien.
- Recherche et insertion : recherche auteur/titre/année/clé, citation simple et groupée, pages et modification/annulation sans perte.

**Acceptation** : projet de test avec deux bibliographies différentes et clés communes ; de l'insertion au PDF, source effectivement citée et bibliographie identiques. Tester notes, auteurs multiples, clés non ASCII et manquantes, conflits et compilation en erreur. Observer le texte extrait et les pages du PDF, pas seulement le succès du compilateur.

## R4 — `0.3.0-alpha.6` : styles CSL et Zotero fiables

**Livrables**
- Choix d'un style Typst intégré ou import `.csl` local, validation, aperçu citation + bibliographie et persistance dans le projet ; règles de priorité explicites face aux directives Typst existantes.
- Zotero en lecture seule : pagination correcte, fidélité des métadonnées, synchronisation rattachée au projet demandeur, clés stables et transaction de tous les fichiers produits ; alternative d'import BibLaTeX sans plugin.
- Styles installés dans Zotero : sélectionner et copier les ressources requises dans le projet. Détecter styles dépendants ; résoudre parent/locales via ressources locales ou import assisté, proposer téléchargement seulement avec consentement. Style manquant = diagnostic, aucun remplacement discret.

**Acceptation** : style intégré, indépendant importé, installé par Zotero et dépendant ; redémarrage hors ligne, parent absent, fichier invalide, changement de projet pendant une requête, coupure Zotero, échec de stockage. Comparer rendus PDF auteur-date, numérique et notes, avec localisateurs et références multiples. Vérifier le moteur verrouillé, signaler les écarts CSL connus.

## R5 — `0.3.0-alpha.7` : devoir et thèse prêts à remettre

**Livrables**
- Modèles de documents utilisables en un geste : devoir court et travail long (métadonnées, page de titre, sections, sommaire, bibliographie et pages) ; normes applicables sans imposer une seule institution.
- Typographie et titres : édition cohérente par niveau réellement appliquée, aperçu et validation ; en-têtes/pieds et champs dynamiques uniquement si le moteur peut les résoudre fidèlement.
- Notes, références croisées, figures et tableaux accessibles : insertion, navigation, mise à jour et diagnostics ; vérifier le comportement et l'utilisation de `cetz` avant de présenter les graphiques comme fiables.

**Acceptation** : produire un devoir synthétique et une thèse synthétique, vérifier titres, sommaire, sauts, notes, légendes, références croisées, bibliographie et export. Réouverture/migration de projets existants sans modifier leur rendu sans choix explicite.

## R6 — `0.3.0-alpha.8` : édition quotidienne et qualité d'interface

**Livrables**
- Consolider onglets/panneaux, focus, gestes souris et clavier, listes longues, sélection de notes et bibliographie ; éviter que la complexité de l'espace masque l'écriture simple.
- Recherche/remplacement littéral et regex exacts, aperçu des changements, undo ; corriger raccourcis, diagnostics, menus et aide, sans conflit avec la saisie.
- Polir thèmes, couleur et contraste, navigation clavier et compatibilité petites fenêtres ; tester les aperçus live/annulation sans changer des réglages enregistrés prématurément.

**Acceptation** : parcours applicatif réel sur trois tailles de fenêtre et thèmes clair/sombre, clavier/souris ; recherche avec métacaractères et `$`, changement d'onglets, suppression/undo de notes, sélection de thème, liste bibliographique longue. Reproduire les bugs observés avant correction et contrôler leur absence après.

## R7 — `0.3.0-alpha.9` : trois plateformes et première installation

**Livrables**
- Abstractions portables pour données, verrou, polices, ouverture de liens/dossiers et découverte Zotero ; fonctionnalités utiles même sans Zotero ni serveur de langage.
- CI et paquets Linux, Windows et macOS : builds et tests natifs, installation, désinstallation, mécanisme de mise à jour et intégrité ; signature/notarisation selon possibilités et contraintes réelles.
- Parcours de première utilisation : créer un projet, choisir un modèle, écrire, ajouter une source et exporter ; aide embarquée, erreurs actionnables et chemin des données visible.

**Acceptation** : sur machines ou VM natives et profils vierges, installation → devoir avec citation → PDF → redémarrage → récupération. Répéter sans réseau, sans Zotero ni tinymist, et avec police absente. Éprouver verrous et remplacement atomique sur chaque système. Aucune promesse de support avant résultats réels.

## R8 — `0.3.0-alpha.10` : interopérabilité et performance contrôlées

**Livrables**
- Export de projet portable et import de sauvegarde : source, normes, styles, bibliographies et ressources, inventaire des dépendances et contrôle des collisions ; ne jamais écraser sans choix.
- Expérimentation DOCX/Markdown : mesurer conversion, tableaux, notes, citations, images et round-trip sur des exemples représentatifs ; si utile et suffisamment fiable, livrer un import ou export annoncé avec ses limites, sinon documenter l'alternative. Le PDF reste référence.
- Mesure de stabilité/performances sur long document et bibliothèque volumineuse, mémoire et rendu incrémental ; améliorer les goulets prouvés, pas des chiffres devinés.

**Acceptation** : transférer un projet entre les trois systèmes et obtenir un PDF comparable ; import interrompu récupérable ; rapport de pertes DOCX ; mesures reproductibles (machine, jeu de test, version, procédure) et absence de pertes à la réouverture.

## R9 — `0.3.0-beta.1` : première bêta

Cette release livre au moins deux évolutions transversales : **centre d'aide/diagnostic et récupération intégrés** finalisés, et **paquets/installateurs + documentation/site accessibles** sur les trois systèmes. Elle n'est ouverte que si les étapes R1–R8 sont effectivement acceptées, y compris la décision DOCX documentée, et si :

- un utilisateur vierge produit et récupère un devoir, tandis qu'une thèse synthétique de longue durée couvre notes et styles CSL ; aucune donnée personnelle réelle n'est nécessaire aux tests ;
- les tests de sûreté, de panne, de compilation et de fidélité bibliographique passent ; zéro bug connu pouvant détruire silencieusement un document, et aucune divergence non signalée entre références visibles et PDF ;
- les CSL intégré, local et Zotero fonctionnent, leur origine est conservée, l'absence de réseau/parent est gérée et les écarts connus sont documentés ;
- les parcours natifs ont été vérifiés et documentés sur Linux, Windows et macOS ; sauvegarde/restauration et migration sont expliquées ;
- les fonctionnalités expérimentales et les dépendances extérieures sont nommées, les limites honnêtes et l'accessibilité minimale clavier/contraste testée ;
- un groupe d'étudiants volontaires peut remonter des incidents, avec procédure de tri, sauvegarde des données et politique de correction ; aucune télémétrie imposée.

La bêta permet des retours et des correctifs, mais ne signifie pas gel définitif. Toute feature après cette porte doit préserver les données et le contrat de rendu.

## Suivi et arbitrages encore ouverts

- DOCX obligatoire ou non : décider après le prototype et les tests de fidélité, pas par défaut.
- Distribution macOS/Windows : choisir et financer les options de signature après étude, sans reporter les builds et essais multiplateformes.
- Définir les exemples bibliographiques réels (notamment sources théologiques) avec des fixtures non personnelles et l'avis d'utilisateurs ; ne pas imposer un style particulier.
- Prioriser toute régression déjà observée par l'utilisateur et conserver l'historique des rapports dans Git ; un « DONE » ancien n'est pas une preuve de validation.
