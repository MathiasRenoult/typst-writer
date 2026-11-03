# Étude de marché et conception bibliographique

Recherche documentaire du 1er novembre 2026. Les comparaisons ci-dessous reposent sur les documentations officielles, pas sur un essai comparatif complet ni sur une enquête utilisateur. Une fonction documentée ailleurs ne démontre pas sa qualité pratique ni sa compatibilité avec la version actuelle de Typst Writer. [Audit du dépôt](audit-avant-beta-2026-11.md) ; [feuille de route](roadmap-vers-beta-2026-11.md).

## Positionnement proposé

Pour un étudiant, le contrat devrait être : « j'écris un texte local lisible, je choisis des sources et un style, je vois le même document que celui exporté, et je peux récupérer mon travail ». Pour la thèse : le même contrat, avec des documents longs, des localisateurs, plusieurs références par note, des sources complexes et des styles institutionnels. L'application est un éditeur de rédaction avec composition Typst, pas un traitement de texte universel et pas un remplaçant de Zotero.

L'avantage à construire : flux simple du Markdown adapté au PDF académique, fidélité des citations, normes réutilisables, travail hors ligne, et résultats contrôlables. Un nombre élevé de boutons ou de thèmes ne compense pas un PDF trompeur ou une référence perdue.

## Panorama : reprendre les gestes, pas copier les interfaces

| Outil | Ce que la documentation établit | Enseignement pour Typst Writer |
|---|---|---|
| Word | Gestion des sources, citations et bibliographies à partir d'une liste de sources.[12] | Reprendre choix de source et changement global de style ; ne pas prétendre qu'un document importé avec ses champs Word reste automatiquement dynamique en Markdown. |
| LibreOffice Writer | Références issues d'une base bibliographique ou conservées dans le document.[13] | Un projet transportable doit contenir les informations nécessaires au PDF, avec provenance et résolution des références. |
| Zotero avec Word/LibreOffice/Google Docs | Ses extensions mettent à jour citations et bibliographie à partir de la bibliothèque.[15] | Un seul état des sources et une mise à jour prévisible ; afficher clairement quand la source n'est plus synchronisée. |
| Zotero lui-même | Gestionnaire de styles, import CSL, aperçu ; le dépôt propose plus de 10 000 styles selon sa documentation.[1][4] | Reprendre sélection, aperçu et diagnostics, sans imposer l'installation de Zotero pour un fichier `.bib`/`.csl`. |
| Typst | Bibliographies BibLaTeX/Hayagriva, styles inclus ou fichiers CSL ; les œuvres citées apparaissent par défaut, `full` permet d'inclure les autres.[2] | Utiliser le même moteur Typst pour l'aperçu et l'export ; exposer le choix du style et des œuvres listées. Les options documentées aujourd'hui doivent être vérifiées sur la version verrouillée, pas supposées identiques. |
| Quarto et Pandoc | Quarto produit des PDF Typst et accepte un style intégré ou un fichier CSL ; le mode visuel propose des outils pour les citations.[3][8] Pandoc convertit différents formats et possède son propre traitement des citations.[7] | Reprendre les fichiers transportables et les choix explicites ; ne pas mélanger silencieusement les moteurs de citation ou promettre un rendu identique entre moteurs. |
| Zettlr | Charge des bibliographies locales, autocomplète et prévisualise ; son guide Zotero utilise Better BibTeX pour un export maintenu à jour.[17][18] Il précise que l'aperçu de citation utilise un style simplifié différent de l'export.[18] | Très pertinent pour le texte local ; faire mieux en signalant explicitement tout aperçu qui n'est pas rendu par le style final. La synchronisation avec Better BibTeX doit rester facultative. |
| Typora | Son export général et ses intégrations Pandoc sont documentés séparément.[9] | Offrir un export simple, en indiquant son moteur et ses dépendances ; conserver un seul rendu PDF faisant autorité. |
| Obsidian | Export PDF de notes sur ordinateur.[14] | Reprendre le geste simple d'export ; ne pas transformer la bêta en système complet de notes et de greffons. |
| Overleaf | Les références sont généralement décrites dans un fichier `.bib`, puis traitées par la chaîne LaTeX du projet.[11] | Reprendre la transparence des sources et des erreurs ; le cloud et LaTeX ne doivent pas devenir obligatoires. |
| Application Web Typst | Édition avec aperçu et synchronisation en temps réel entre coauteurs.[22] | Reprendre la clarté du lien source/rendu ; traiter la collaboration comme une évolution distincte, non comme une obligation de connexion. |
| JabRef | Recherche et insertion de clés depuis un éditeur LaTeX, avec JabRef lancé et son serveur HTTP activé.[19] | Séparer gestion des notices et rédaction ; ne pas exiger son serveur pour citer un fichier local. |
| Google Docs | Suggestions de modifications conservant le texte d'origine jusqu'à leur acceptation.[20] | Un mode révision est pertinent à terme pour les corrections d'enseignants ; ne pas le confondre avec une simple liste de commentaires. |
| Word / PDF accessible | Microsoft documente des balises PDF pour lecteurs d'écran, liens, signets et textes alternatifs.[21] | Vérifier les PDF générés avec des outils d'accessibilité et signaler les limites ; un PDF visuellement correct ne suffit pas. |

Échanges DOCX : Pandoc sait convertir plusieurs formats, dont DOCX et Markdown.[7] C'est une piste et non la preuve d'un aller-retour fidèle des styles, notes, citations et tableaux. Leur caractère obligatoire pour la bêta reste indécidé ; proposer une expérimentation sur documents synthétiques, rapport de pertes et rejet explicite des conversions destructives.

## Ce que doit faire le parcours bibliographique

### 1. Bibliothèque et identité des références

- Une bibliothèque locale `.bib` lisible et éditable ; import sans modifier l'original, et copie ou lien explicite avec leur statut de synchronisation.
- Zotero facultatif, en lecture seule : choix de bibliothèque/collection, recherche, prévisualisation, import synchronisé. Son API locale fonctionne hors ligne et doit être activée dans Zotero ; les lectures sont sans authentification et le port ne doit pas être exposé au réseau.[5]
- Une identité stable par source et clé, des collisions explicites, pas de renommage silencieux. Préserver rôles d'auteurs, directeurs et traducteurs, date complète, langues, sous-titres, chapitres, volumes, pages, DOI/URL, éditions et casse protégée. Règle de round-trip sur jeux de notices ; signaler toute perte inévitable.
- Une source de vérité partagée par liste, complétion, citations, aperçu et export ; affichage du dernier état synchronisé et possibilités de correction. Pas d'écriture directe dans Zotero.

### 2. Style du projet : trois portes d'entrée

1. Choisir un style intégré à la version précise de Typst embarquée, avec nom humain, type et aperçu ; ne pas maintenir à la main une liste qui diverge de cette version.
2. Importer un `.csl` local ; conserver l'original et une copie/version identifiée dans le projet afin que le style suive celui-ci hors ligne. Contrôler la validité et indiquer précisément ce que le moteur accepte.
3. Choisir parmi les styles installés dans le Zotero local ; lire leurs fichiers, afficher leur provenance, copier les ressources nécessaires dans le projet sans modifier Zotero. Un Zotero absent ou éteint ne doit pas rendre le PDF impossible.

Le choix doit régir citations et bibliographie ensemble. Un style imposé par un fichier Typst personnalisé ne doit pas être écrasé silencieusement : afficher lequel fait autorité et proposer une résolution explicite. Changer un style ne doit jamais modifier les clés ou les notices. Prévisualiser un petit jeu représentatif, puis le vrai document ; rétablir le choix précédent si validation annulée ou fichier invalide.

### 3. CSL dépendants, locales et dépendances

La spécification CSL distingue styles indépendants et dépendants : un dépendant référence un parent indépendant et peut préciser une locale.[6] Un fichier choisi ne suffit donc pas toujours ; vérifier sur le moteur précis embarqué ce qu'il accepte. Au minimum : détecter le parent, chercher parmi les styles installés ou une source locale, proposer l'import du parent ; si nécessaire proposer un téléchargement explicite, jamais imposé ni silencieux. Conserver le nom demandé, la locale et les versions des deux fichiers, sans écraser les fichiers utilisateurs. Si le parent manque, expliquer le geste à effectuer et préserver le dernier PDF valide.

Gérer la locale du document, du style et des références comme des données distinctes ; tester français, anglais et cas plurilingues. Traiter les styles CSL locaux comme des entrées non fiables : tailles bornées, XML/CSL validé, droits limités, absence d'exécution arbitraire et messages d'erreur. La validation et les noms de fichiers ne prouvent pas la conformité du rendu.

### 4. Citation au fil de l'écriture

- Recherche auteur/titre/année/clé, insertion clavier et souris, résultat lisible par défaut, accès rapide aux notices choisies ; une citation existante s'édite sans perdre le contenu en cas d'Échap.
- Localisateurs typés ou libres (page, chapitre, volume, folio), préfixe/suffixe et suppression d'auteur si le moteur le permet ; plusieurs références par appel et ordre conservé. Citer un ouvrage dans le texte comme auteur ou en note selon le style, pas par une hypothèse de présentation codée en dur.
- Bibliographie finale configurée (toutes œuvres ou citées, titre, position, langue), cohérente avec les citations. Les besoins théologiques spécifiques (sources anciennes, auteurs collectifs, traductions, éditions et références scripturaires) nécessitent une étude avec exemples : ne pas inventer une norme.
- Une référence introuvable, une conversion incomplète ou un fichier manquant doit se voir et être corrigeable ; ne pas remplacer une citation par un texte apparemment ordinaire pour faire disparaître l'erreur.

### 5. Qualité : rendre la conformité mesurable

Constituer des fixtures publiques/synthétiques et un jeu de sorties de référence vérifiées humainement : auteur-date, numérique, notes, citations successives et multiples, localisateurs, langues, styles personnalisés et dépendants, incohérences de source, reprise hors ligne. Comparer texte et structure du PDF, pas seulement compilation réussie ; relever les écarts entre moteur embarqué et Zotero/citeproc-js sans postuler l'égalité. Tester dans les trois systèmes et sur une copie de projet ; aucun document personnel dans les fixtures ou les logs.

## Choix délibérés pour la bêta

- Hors ligne et accès à ses fichiers : obligatoires ; partage cloud, collaboration temps réel, comptes obligatoires et écriture dans Zotero : hors périmètre bêta recommandé.
- Plusieurs formats d'import ou d'export : utiles, mais un aller-retour Word sans pertes n'est pas prérequis établi. Décider après mesures et consultation.
- Styles livrés : préférer le catalogue réellement intégré au moteur, puis les CSL de l'utilisateur ; ne pas livrer une compilation des styles du dépôt sans examiner licence, versionnement et mise à jour.
- Privilégier installation, récupération et erreurs actionnables aux fonctions spectaculaires. Une option avancée doit avoir un propriétaire et un test, pas seulement une case.

## Sources

[1] https://www.zotero.org/support/styles
[2] https://www.typst.app/docs/reference/model/bibliography
[3] https://quarto.org/docs/output-formats/typst.html
[4] https://www.zotero.org/support/preferences/cite
[5] https://www.zotero.org/support/dev/web_api/v3/local_api
[6] https://docs.citationstyles.org/en/stable/specification.html
[7] https://pandoc.org/MANUAL.html
[8] https://quarto.org/docs/visual-editor/technical.html
[9] https://support.typora.io/Export
[11] https://www.overleaf.com/learn/how-to/Using_bibliographies_on_Overleaf
[12] https://support.microsoft.com/en-us/word/add-or-change-sources-citations-and-bibliographies
[13] https://help.libreoffice.org/latest/en-US/text/swriter/guide/indices_literature.html
[14] https://help.obsidian.md/Export+to+PDF
[15] https://www.zotero.org/support/word_processor_integration
[17] https://docs.zettlr.com/en/guides/reference-manager-integration.html
[18] https://docs.zettlr.com/en/editor/citations.html
[19] https://docs.jabref.org/cite/cite-as-you-write
[20] https://support.google.com/docs/answer/6033474?co=genie.platform%3Ddesktop&hl=en
[21] https://support.microsoft.com/en-us/office/064625e0-56ea-4e16-ad71-3aa33bb4b7ed
[22] https://typst.app/docs/web-app
