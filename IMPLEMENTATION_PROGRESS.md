# Bug report 5 — Suivi d'implémentation

| Identifiant | État | Fichiers | Tests | Notes |
|---|---|---|---|---|
| LEFT-04-01 — rouvrir le panneau | DONE | ui/toolbar.rs | 395 | Bouton dans la barre |
| LEFT-04-02 — supprimer onglet Documents | DONE | ui/sidebar.rs | 395 | Section supprimée |
| LEFT-04-03 — déplacer nom du document | DONE | ui/sidebar.rs | 395 | Nom en en-tête |
| LEFT-04-04 — alignement projets | DONE | ui/sidebar.rs | 395 | row! align_y(Center) |
| LEFT-04-05 — alignement bouton Nouveau projet | DONE | ui/sidebar.rs | 395 | Déjà centré |
| TOPBAR-04-01 — réorganiser groupes | DONE | ui/toolbar.rs | 395 | 2e spacer |
| TOPBAR-04-02 — supprimer commandes fichier | DONE | ui/toolbar.rs | 395 | NewFile, Bibliothèque, Export |
| TOPBAR-04-03 — centrer icônes format | DONE | ui/toolbar.rs | 395 | container center_x/y |
| TOPBAR-04-04 — menu titres utilisable | DONE | ui/toolbar.rs | 395 | pick_list |
| TOPBAR-04-05 — retirer choix police | DONE | ui/toolbar.rs | 395 | Supprimé |
| TOPBAR-04-06 — pastille couleur visible | DONE | ui/toolbar.rs | 395 | Valeurs hex |
| TOPBAR-04-07 — synchroniser couleur courante | DONE | app/mod.rs | 395 | derniere_couleur |
| TOPBAR-04-08 — améliorer color picker | DONE | ui/toolbar.rs | 395 | Palette + picker RVB thème |
| TOPBAR-04-09 — supprimer commandes aperçu | DONE | ui/toolbar.rs | 395 | Supprimées |
| TOPBAR-04-10 — replacer commande normes | DONE | ui/toolbar.rs | 395 | Tout à gauche |
| THEME-EDITOR-04-01 — color picker intégré | DONE | ui/theme_editeur.rs | 395 | Picker RVB, zenity supprimé |
| THEME-EDITOR-04-02 — supprimer pastilles b/n | DONE | ui/theme_editeur.rs | 395 | Supprimées |
| EDITOR-04-01 — supprimer glyphe complet | DONE | vendor deletion.rs | 395+654 | has_selection() |
| EDITOR-04-02 — fermer overlay au clic extérieur | DONE | ui/mod.rs | 395 | mouse_area |
| EDITOR-04-03 — rendu dernières lignes | DONE | vendor canvas.rs | 395+654 | min/max plage visible |
| NORMS-04-01 — flèches sur champs non numériques | DONE | ui/reglages.rs | 395 | saisie_numerique |
| NORMS-04-02 — config par niveau de titre | DONE | mise_en_page/modele.rs, gabarit.rs | 395 | couleur/gras/espacement/casse par niveau |
| NORMS-04-03 — déplacer gestion modèles | DONE | ui/reglages.rs | 395 | En haut de la fenêtre |
