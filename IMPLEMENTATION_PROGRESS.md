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
| LEFT-05 — toutes les collections dans le sélecteur | TODO | | | |
| TOPBAR-05 — menu des titres utilisable | TODO | | | |
| THEME-EDITOR-01 — édition des valeurs hex | DONE | state.rs, theme_editeur.rs | editeur_theme_textes | Saisie brute préservée |
| WINDOWS-01 — focus entre fenêtres | TODO | | | |
| WINDOWS-02 — règles de clic pour ouvrir | TODO | | | |
| NORMS-01 — chevauchement barre de défilement | TODO | | | |
| NORMS-02 — prévisualisation au mauvais endroit | TODO | | | |
| FIND-03 — dépassement des placeholders | TODO | | | |

## Lot 3 — Éditeur et rendu (P1+P2)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| EDITOR-01 — couleur de fond unifiée | TODO | | | |
| EDITOR-02 — Ctrl+Suppr supprime un mot | TODO | | | |
| RENDER-02 — netteté du rendu | TODO | | | |
| RENDER-01 — limite de zoom | TODO | | | |

## Lot 4 — Organisation de l'interface (P2)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| LEFT-01 — déplacer le bouton de réduction | TODO | | | |
| LEFT-02 — supprimer le bloc redondant | TODO | | | |
| LEFT-03 — supprimer les anciens chapitres | TODO | | | |
| LEFT-04 — sommaire en arborescence | TODO | | | |
| LEFT-06 — intitulé bibliographie | TODO | | | |
| LEFT-07 — comptage des entrées | TODO | | | |
| LEFT-08 — année de publication | TODO | | | |
| LEFT-09 — filtres de bibliothèque | TODO | | | |
| LEFT-10 — onglet Document | TODO | | | |
| LEFT-11 — actions projet au survol | TODO | | | |
| TOPBAR-01 — supprimer nom technique | TODO | | | |
| TOPBAR-02 — réorganiser les boutons | TODO | | | |
| TOPBAR-04 — alignement icônes | TODO | | | |
| TOPBAR-06 — commande de couleur | TODO | | | |
| TOPBAR-07 — boutons d'aperçu obsolètes | TODO | | | |
| TOPBAR-08 — commandes de zoom | TODO | | | |
| TOPBAR-09 — thèmes à droite | TODO | | | |
| TOPBAR-10 — supprimer commandes thèmes | TODO | | | |
| TOPBAR-11 — thèmes favoris | TODO | | | |
| TOPBAR-12 — bouton bibliothèque | TODO | | | |
| TOPBAR-13 — éditeur de thèmes avec thèmes | TODO | | | |
| FOCUS-01 — masquer barre inférieure | TODO | | | |
| FOCUS-02 — centrer le rendu | TODO | | | |
| NORMS-03 — curseur interactif | TODO | | | |
| NORMS-05 — commandes en haut | TODO | | | |
| THEME-EDITOR-02 — color pickers | TODO | | | |
| THEME-EDITOR-03 — pastilles de couleur | TODO | | | |
| THEME-EDITOR-05 — polir l'éditeur de thèmes | TODO | | | |

## Lot 5 — Fonctions éditoriales et typographiques (P2+P3)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| EDITOR-03 — `---` séparateur horizontal | TODO | | | |
| EDITOR-04 — trois lignes vides = saut de page | TODO | | | |
| EDITOR-05 — autocomplétion tableaux | TODO | | | |
| EDITOR-06 — souligné et barré | TODO | | | |
| EDITOR-07 — commentaires | TODO | | | |
| EDITOR-08 — petites capitales siècles | TODO | | | |
| EDITOR-09 — barre de défilement discrète | TODO | | | |
| EDITOR-10 — aides visuelles | TODO | | | |
| NORMS-04 — config des niveaux de titre | TODO | | | |
| NORMS-07 — guillemets par niveau | TODO | | | |
| NORMS-08 — ordre guillemets français | TODO | | | |
| NORMS-09 — normes allemandes | TODO | | | |
| NORMS-10 — séparateurs horizontaux | TODO | | | |
| NORMS-11 — Quick Parts | TODO | | | |

## Lot 6 — Import et recherche avancée (P2+P3)

| Demande | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| TOPBAR-03 — panneau d'import | TODO | | | |
| FIND-01 — choix normal/regex | TODO | | | |
| FIND-02 — mise en évidence | TODO | | | |
| WINDOWS-03 — aperçu glisser-déposer | TODO | | | |
| WINDOWS-04 — détachement d'onglet | TODO | | | |
