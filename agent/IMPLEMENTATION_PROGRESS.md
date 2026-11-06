# bug-report-10 — état d'implémentation

Feuille de route par lots, complexité croissante. Un lot = vérifié par
`cargo test` + `clippy -D warnings`, tracé au suivi de développement.
Aucun commit sans demande. L'écran revient à l'auteur : chaque point
livré distingue ce que les tests couvrent de ce qu'il doit voir dans
l'app.

## 1. Normes et structure documentaire

- **1.1 Saut de page avant la bibliographie** — FAIT. L'option des
  normes (onglet Bibliographie) pose `#pagebreak()` avant l'appel
  `#bibliography`.
- **1.2 Bibliographie en titre de niveau 1** — FAIT. La même option
  compose la bibliographie sous un titre `= Bibliographie`, la liste
  perdant son titre propre (`title: none`).
- **1.3 Balise `[[TOC]]` + bouton** — FAIT. La balise de la langue
  `.tw` génère la table des matières (`#outline()`, équivalent de
  `{glossaire}`), insérable par le bouton de la barre.

## 2. Left panel

- **2.1 Sommaire : espacement des lignes hiérarchiques** — FAIT. Les rangées se touchent (colonne à espacement nul) : les lignes dessinées sont continues, sans espace aux bordures ; l'indentation est conservée, les barres s'allongent d'une rangée à l'autre.
  Lignes visuellement continues, sans espace aux bordures du panneau ;
  indentation conservée, lignes allongées.
- **2.2 Bibliographie : repli des entrées + recherche** — FAIT. Les
  entrées individuelles se replient (chevron, une ligne repliée) ; la
  recherche montre ses résultats même la section repliée.
- **2.3 Projets + bibliothèque** — FAIT. La section Projets avait déjà
  les gestes (vérifiés au code : « Nouveau projet », crayon = renommer,
  poubelle = supprimer) et le choix d’ étagère ; il lui manquait
  l’accès à la bibliothèque — un bouton (icône bibliothèque) ouvre la
  vue étagères depuis la section.

## 3. Listes à plusieurs niveaux

- **3.1–3.6 Listes multiniveaux** — FAIT. **La langue est faite**
  (3.1, 3.5) : un élément de liste porte son niveau (`ElementListe`,
  deux espaces de retrait par niveau) — parseur, générateur (retrait
  Typst), sérialiseur `.tw`, preuve `une_liste_multiniveau_se_lit`.
  **Les gestes sont faits** (3.2, 3.3, 3.6) : sur une ligne de liste,
  Tab augmente le niveau, Maj+Tab ou Ctrl+Tab le diminue, et
  Ctrl+Maj+Haut/Bas déplace l'**élément** (le bloc reste aux autres
  lignes) — le curseur suit sa ligne. Preuves :
  `le_curseur_reconnait_une_liste`, `le_niveau_monte_et_redescend`,
  `l_element_se_deplace`. L'autocomplétion est faite (3.4) :
  `Entrée` sur un élément crée le suivant avec sa marque — la puce
  reste, le numéro avance — preuve `la_suite_de_la_liste_se_continue`.
  Le lot 7 est complet.
  plusieurs niveaux ; Tab / Ctrl+Tab ou Shift+Tab pour le niveau ;
  autocomplétion `1. ` + Entrée qui enchaîne ; niveaux des listes
  numérotées ; Ctrl+Shift+↑/↓ pour déplacer un élément.

## 4. Métadonnées et profils

- **4.1–4.5 Profils de métadonnées** — FAIT. Profils nommés portant
  leurs valeurs (4.1, 4.2) ; les balises `{clé}` lisent le profil actif
  (4.3) ; « Remplacer les balises du texte par leurs valeurs » (4.4) ;
  clés système réservées et refusées avec message (4.5). Le panneau des
  métadonnées a la rangée de profils (menu, nom, enregistrer, supprimer)
  et le bouton de remplacement. Preuves : `les_profils_portent_leurs_valeurs`,
  `les_cles_systeme_sont_reservees`, `les_balises_se_remplacent_par_leurs_valeurs`.
  métadonnée porte une valeur par profil ; les balises utilisent le
  profil actif ; remplacement automatique des balises par leurs valeurs ;
  noms de balises système réservés.

## 5. Top bar et ergonomie

- **5.1 BIB / GLO en icônes** — FAIT. `icon_library` et `icon_book_a`, infobulles gardées.
- **5.2 Styles de titres : icône + menu** — FAIT. `icon_heading` ouvre un menu `iced_aw` Titre 1..6 (le modèle du menu « Projet »).
- **5.3 Glossaire : icône + fenêtre dédiée (consulter, ajouter,
  modifier, supprimer)** — FAIT. `icon_book_marked` ouvre `src/ui/glossaire_fenetre.rs` : liste filtrable, Insérer / Modifier / Supprimer / Ajouter / Fermer. Épreuve `la_fenetre_du_glossaire_s_ouvre_et_se_ferme`.
- **5.4 Couleurs : choix personnalisé vs application, dernière couleur
  accessible** — FAIT. Le sélecteur a « Enregistrer » (le choix : la dernière couleur, rien inséré) et « Appliquer » (l'application) — épreuve `le_choix_de_couleur_n_insere_rien`. La pastille insère la balise de la dernière couleur.
- **5.5 Ligne de séparation entre fenêtres** — FAIT. Le réseau de panneaux en espacement 1 px sur fond de bordure : une fine ligne entre les fenêtres.

## 6. Rendu et compilation

- **6.1 Bibliographies dupliquées : erreurs de compilation** — FAIT.
  **Cause (sonde Typst à l'appui)** : deux `.bib` fondus dans un appel
  qui partagent une clé lèvent `duplicate bibliography keys` — et la
  guérison des bibliographies ne voyait que le **premier** chemin entre
  guillemets d'un appel multiple, donc jamais le second fichier. Deux
  correctifs : le balayage cite **tous** les chemins de l'appel, et la
  guérison **retire les entrées dupliquées** (la bibliothèque locale
  gagne sur l'export Zotero, qui ne se réécrit pas). Preuve par le
  vrai moteur : `les_cles_dupliquees_ne_cassent_plus_le_rendu`.
- **6.2 Synchro édition / rendu : actualisation fluide et cohérente** —
  FAIT. **Deux mécanismes** : (a) **le battement du rendu** — une frappe
  continue ne reste jamais plus de 2,5 s sans nouvelle (auparavant, le
  repos de 2 s se repoussait à chaque frappe : taper longtemps ne
  rendait jamais) ; (b) **le basculement compile d'entrée** — passer en
  vue Rendu avec un texte périmé lance la compilation immédiatement,
  jamais l'image d'une frappe d'avant. Preuves :
  `le_battement_rend_les_frappes_continues`,
  `la_frappe_ne_compile_pas_avant_le_battement` (la règle « la frappe
  ne compile jamais » est remplacée par sa forme exacte : jamais par
  frappe, sauf le battement).

---

## Lots et vérifications

### Lot 1 (5.1, 5.2, 5.5) — Fait et vérifié
Barre : icônes BIB/GLO, menu de titres à icône, ligne entre fenêtres.
`cargo test` 541 verts, `clippy -D warnings` vert. Reste à voir dans
l'app : le rendu des icônes, l'ouverture du menu, la ligne entre panneaux.

### Lot 2 (5.4) — Fait et vérifié
Choix (« Enregistrer ») et application (« Appliquer ») distincts dans le
sélecteur de couleur ; la couleur appliquée devient la dernière ;
l'accès rapide par la pastille est conservé. Épreuve
`le_choix_de_couleur_n_insere_rien` verte. Reste à voir : la tenue du
dialogue.

### Lot 5, partie 1 (2.1, 2.2) — Fait et vérifié
Sommaire : lignes hiérarchiques continues (rangées jointives, barres
allongées). Bibliographie : entrées repliables individuellement, et la
recherche montre ses résultats même section repliée. 543 tests, clippy
vert. Reste à voir : la continuité des lignes, le chevron de repli, la
recherche en section repliée.

### Lot 6 (4.1–4.5) — Fait et vérifié
Profils de métadonnées, réservation des clés système, remplacement des
balises — couche de données, traitements et panneau. 546 tests, clippy
vert. Reste à voir : la rangée de profils, le refus d'une clé réservée,
le bouton de remplacement.

### Lot 5, partie 2 (2.3) — Fait et vérifié
Vérifié au code d’ abord : créer / renommer / supprimer existaient déjà
sur les rangées de projet, et l’ étagère se choisissait déjà au menu —
la demande est un constat d’ état, pas une réécriture. Ajouté : l’accès
à la bibliothèque (vue étagères) depuis la section, par un bouton à
icône. 543 tests, clippy vert. Reste à voir : le bouton et la vue
étagères qu’il ouvre.
Projets : intégrer la bibliothèque (vue étagères, `ProjetsEtageChoisie`)
et permettre créer / renommer / supprimer des projets depuis la section.

### Lot 4 (1.1, 1.2, 1.3) — Fait et vérifié
Normes de placement de la bibliographie (onglet Bibliographie) : saut de
page avant, titre de niveau 1 — appliquées à l' + q + 'assemblage, sans
double titre. Balise `[[TOC]]` de la langue (parseur, schéma,
sérialiseur, générateur `#outline()`) et son bouton dans la barre.
Preuves : `la_bibliographie_se_place_selon_les_normes`,
`la_ligne_toc_est_un_bloc`. 543 tests, clippy vert, 126/126 épreuves.
Reste à voir : les cases du panneau, le bouton, le rendu du titre et du
saut de page.

### Lot 3 (5.3) — Fait et vérifié
Fenêtre du glossaire (`src/ui/glossaire_fenetre.rs`) : consulter,
chercher, insérer au curseur, ajouter, modifier, supprimer — les gestes
existants y sont réunis, la liste déroulante de la barre est remplacée
par son icône. Épreuve `la_fenetre_du_glossaire_s_ouvre_et_se_ferme`
verte. 541 tests, clippy vert. Reste à voir : la fenêtre elle-même.
