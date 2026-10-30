# Suivi d'implémentation — bug-report-4

Cahier des charges : `agent/bug-report-4.md`
Ordre de traitement : P0 d'abord, puis P1, P2, P3, par lots cohérents.

## Lot 1 — Sécurité des données (P0)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| THEME-01 — perte de thème à l'enregistrement | DONE | theming/mod.rs | identical_background_and_surface_warn | Contraste = avertissement, pas refus |
| THEME-EDITOR-04 — corruption de thème | DONE | theming/mod.rs | validation dans enregistrer() | Erreurs bloquent, avertissements passent |
| LEFT-12 — fiabiliser le renommage de projet | DONE | app/mod.rs, project/mod.rs | verrou libéré avant rename | Plus de faux conflit, nom propagé |
| NORMS-06 — norme active et état de modification | DONE | state.rs, reglages.rs | norme_originale + modifie | Suivi d'état posé |

## Lot 2 — Bugs d'interaction (P1)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| LEFT-05 — toutes les collections dans le sélecteur | DONE | ui/sidebar.rs | Hauteur contrainte |
| TOPBAR-05 — menu des titres utilisable | DONE | ui/toolbar.rs | Menu reste ouvert |
| THEME-EDITOR-01 — édition des valeurs hex | DONE | state.rs, theme_editeur.rs | editeur_theme_textes | Saisie brute préservée |
| WINDOWS-01 — focus entre fenêtres | DONE | app/mod.rs, espace_runtime.rs | Ctrl+Maj+Tab bascule de panneau |
| WINDOWS-02 — règles de clic | DONE | app/espace_runtime.rs | Clic=Remplacer (bascule si ouvert), Maj+clic=Nouvel (toujours) |
| NORMS-01 — chevauchement barre de défilement | DONE | ui/reglages.rs | Correction |
| NORMS-02 — prévisualisation au mauvais endroit | DONE | ui/reglages.rs | Correction |
| FIND-03 — dépassement des placeholders | DONE | ui/recherche.rs | Length::Fill au lieu de Fixed | |

## Lot 3 — Éditeur et rendu (P1+P2)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| EDITOR-01 — couleur de fond unifiée | DONE | ui/theme.rs | editor.background = colors.background | Déjà résolu par le code existant |
| EDITOR-02 — Ctrl+Suppr supprime un mot | DONE | vendor/iced-code-editor | DeleteWord + handle_delete_word | PATCHES.md point 16 |
| RENDER-02 — netteté du rendu | DONE | ui/preview.rs | MAX_DISPLAY_RATIO=5.0 |
| RENDER-01 — limite de zoom | DONE | ui/preview.rs | MAX_DISPLAY_RATIO = 5.0 | 500% au lieu de 200% |

## Lot 4 — Organisation de l'interface (P2)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| LEFT-01 — déplacer bouton réduction | DONE | ui/sidebar.rs | dans le panneau |
| LEFT-02 — supprimer le bloc redondant | DONE | ui/sidebar.rs | Bloc supprimé |
| LEFT-03 — supprimer les anciens chapitres | DONE | ui/sidebar.rs | section retirée | |
| LEFT-04 — sommaire arborescence | DONE | ui/sidebar.rs | Lignes de connexion │ ├── └── |
| LEFT-06 — intitulé bibliographie | DONE | ui/sidebar.rs | « Entrées bibliographiques » |
| LEFT-07 — comptage des entrées | DONE | project/mod.rs | Entrée tronquée ignorée |
| LEFT-08 — année de publication | DONE | ui/sidebar.rs | Année dans la ligne |
| LEFT-09 — filtres de bibliothèque | DONE | ui/sidebar.rs | Année dans le filtre |
| LEFT-10 — onglet Document | DONE | ui/sidebar.rs | « N mots » libellé | |
| LEFT-11 — actions projet au survol | DONE | state.rs, message.rs, sidebar.rs | survol_projet + mouse_area | |
| TOPBAR-01 — supprimer nom technique | DONE | ui/toolbar.rs | document.tw supprimé |
| TOPBAR-02 — réorganiser les boutons | DONE | ui/toolbar.rs | séparateur format/panneaux |
| TOPBAR-04 — aligner icônes | DONE | ui/toolbar.rs | BUTTON=30 pour tous |
| TOPBAR-06 — commande de couleur | DONE | ui/toolbar.rs | Pastille + sélecteur |
| TOPBAR-07 — boutons d'aperçu obsolètes | DONE | ui/toolbar.rs | zoom déplacé (TOPBAR-08), reste est utile |
| TOPBAR-08 — commandes de zoom | DONE | ui/preview.rs, ui/toolbar.rs | zoom bar dans le rendu | |
| TOPBAR-09 — thèmes à droite | DONE | ui/toolbar.rs | espace flexible |
| TOPBAR-10 — supprimer commandes thèmes | DONE | ui/toolbar.rs | boutons retirés | |
| TOPBAR-11 — sélecteur de police | DONE | ui/toolbar.rs | pick_list familles |
| TOPBAR-12 — bouton bibliothèque | DONE | ui/toolbar.rs | parmi les commandes fichier |
| TOPBAR-13 — éditeur thèmes avec thèmes | DONE | ui/toolbar.rs | groupe thème à droite |
| FOCUS-01 — masquer barre inférieure | DONE | ui/mod.rs | status_bar retiré du mode focus | |
| FOCUS-02 — centrer rendu en focus | DONE | ui/layout.rs | container center_x/y |
| NORMS-03 — curseur interactif | DONE | iced 0.14 natif | Les widgets interactifs changent déjà le curseur |
| NORMS-05 — commandes en haut | DONE | ui/reglages.rs | Importer/Exporter en tête |
| THEME-EDITOR-02 — color pickers | TODO | | | |
| THEME-EDITOR-03 — pastilles de couleur | DONE | ui/theme_editeur.rs | tooltip |
| THEME-EDITOR-05 — polir l'éditeur de thèmes | DONE | ui/theme_editeur.rs | sections Fond/Texte/États |

## Lot 5 — Fonctions éditoriales et typographiques (P2+P3)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| EDITOR-03 — `---` séparateur horizontal | DONE | parse.rs, serialiser.rs, generer.rs | Bloc::Separateur | #line(length: 100%) |
| EDITOR-04 — trois lignes vides = saut de page | DONE | langage/{schema,parse,serialiser,generer}.rs | SautDePage, #pagebreak() |
| EDITOR-05 — autocomplétion tableaux | DONE | langage/parse.rs | Autoformatage colonnes |
| EDITOR-06 — souligné et barré | DONE | schema.rs, parse.rs, serialiser.rs, generer.rs | __…__, ~~…~~ | #underline[], #strike[] |
| EDITOR-07 — commentaires | DONE | schema.rs, parse.rs | // texte → Bloc::Commentaire |
| EDITOR-08 — petites capitales siècles | DONE | typographie.rs, generer.rs | petites_capitales_siecles | XXe → #smallcaps[XX]e |
| EDITOR-09 — barre de défilement discrète | DONE | ui/theme.rs | scroller alpha 0.4 | |
| EDITOR-10 — aides visuelles | DONE | vendor/iced-code-editor | Ctrl+L, Ctrl+Maj+L |
| NORMS-04 — config niveaux de titre | DONE | mise_en_page/modele.rs | casse_1 à casse_6 |
| NORMS-07 — guillemets par niveau | DONE | ui/reglages.rs | Aperçu imbrication |
| NORMS-08 — ordre guillemets français | DONE | typographie.rs | «», ‹›, "" |
| NORMS-09 — normes allemandes | DONE | typographie.rs | allemande() | „…", ‚…', »…« |
| NORMS-10 — séparateurs horizontaux | DONE | modele.rs | separateur_couleur/epaisseur |
| NORMS-11 — Quick Parts | DONE | mise_en_page/gabarit.rs | {document} {date} {auteur} {projet} |

## Lot 6 — Import et recherche avancée (P2+P3)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| TOPBAR-03 — panneau d'import | DONE | ui/toolbar.rs | Libellé clarifié |
| FIND-01 — choix littéral/regex | DONE | app/recherche.rs, ui/recherche.rs | bouton Aa/.* |
| FIND-02 — mise en évidence | DONE | app/mod.rs, vendor set_search_query | PATCHES.md point 17 |
| WINDOWS-03 — aperçu glisser-déposer | DONE | ui/layout.rs | Liseré accentué sur la cible |
| WINDOWS-04 — détachement d'onglet | DONE | app/espace_runtime.rs | Dépôt hors panneau = nouveau panneau |
