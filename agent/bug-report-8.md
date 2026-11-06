Voici une décomposition en tâches atomiques, structurée pour qu'Hermes puisse les traiter individuellement, vérifier chaque comportement et éviter de modifier inutilement le reste de l'application.

J'ai regroupé les tâches par section du panneau Normes du document, avec pour chacune un objectif, les modifications attendues et les critères d'acceptation.

# Backlog technique : panneau Normes du document



9 chantiers

20 tâches atomiques



## 1. Onglet Texte

### T1.1 – Corriger le chargement des polices système

- Examiner le mécanisme actuel de récupération des polices installées sur le système.
- Identifier pourquoi certaines polices disponibles sur l'ordinateur ne sont pas proposées.
- Corriger le chargement afin de récupérer l'ensemble des polices réellement disponibles.
- Éviter les listes partielles, aléatoires ou dépendantes de l'ordre de chargement.
- Vérifier le comportement sur plusieurs familles de polices et styles (Regular, Bold, Italic, etc.).

Critère d'acceptation : toutes les polices installées et accessibles au système apparaissent de manière cohérente dans le sélecteur.

### T1.2 – Ajouter la saisie directe du nom de police

- Transformer le sélecteur de police en champ de recherche avec autocomplétion.
- Permettre de saisir directement le nom d'une police, comme dans Microsoft Word.
- Filtrer dynamiquement les polices disponibles pendant la saisie.
- Permettre de sélectionner une police depuis les résultats ou de conserver une saisie valide.

Critère d'acceptation : l'utilisateur peut rechercher et sélectionner une police sans parcourir manuellement toute la liste.

## 2. Onglet Titres

### T2.1 – Ajouter le sélecteur de couleur aux niveaux de titre

- Ajouter une pastille de couleur à chaque niveau de titre.
- Permettre de définir indépendamment la couleur de chaque niveau.
- Au clic sur la pastille, ouvrir le color picker.
- Conserver la couleur sélectionnée dans les normes du document.

Critère d'acceptation : chaque niveau de titre dispose de sa propre couleur configurable.

### T2.2 – Supprimer les sections inutiles

- Supprimer la section « Couleur » située en bas de l'onglet Titres.
- Supprimer la section « Les défauts » située en bas de l'onglet Titres.
- Vérifier que leur suppression ne retire aucune fonctionnalité encore utilisée ailleurs.

### T2.3 – Ajouter un format de numérotation personnalisé

- Conserver les modèles de numérotation prédéfinis.
- Ajouter une option permettant de définir une syntaxe personnalisée.
- S'inspirer du système de formatage personnalisé des cellules dans Excel.
- Permettre de composer librement le format de numérotation des chapitres.
- Prévoir une documentation courte des caractères et variables disponibles.
- Ajouter un aperçu en temps réel du résultat.
- Vérifier que les formats personnalisés sont correctement interprétés lors du rendu Typst.

Critère d'acceptation : l'utilisateur peut choisir un modèle prédéfini ou définir son propre format de numérotation sans modifier le code Typst manuellement.

## 3. Onglet Figures et dates

### T3.1 – Documenter les formats de dates complètes

- Ajouter une courte explication sous le champ « Format des dates pleines ».
- Présenter les caractères disponibles et leur signification.
- Donner quelques exemples de formats.
- Expliquer brièvement comment composer un format personnalisé.

Exemple de documentation attendue :

| Symbole | Signification |
| ------- | ------------- |
| `d`     | Jour          |
| `M`     | Mois          |
| `y`     | Année         |

Les symboles exacts devront correspondre au moteur de formatage réellement utilisé par l'application.

### T3.2 – Ajouter le choix de la langue des dates

- Ajouter un sélecteur de langue pour l'affichage des dates.
- Permettre de choisir la langue indépendamment de la langue de l'interface.
- Prévoir au minimum les langues pertinentes pour l'application : français, anglais, allemand et italien.
- Vérifier que les noms des mois et des jours sont correctement localisés.
- Vérifier la compatibilité avec les formats personnalisés.

Critère d'acceptation : une date peut être affichée dans une langue différente de celle de l'interface, sans modifier le format choisi.

## 4. Onglet En-tête et pied de page

### T4.1 – Corriger le rendu des Quick Parts de métadonnées

- Examiner le système de Quick Parts utilisé dans les en-têtes et pieds de page.
- Vérifier les sept métadonnées actuellement disponibles.
- Identifier pourquoi seul le champ « Auteur » est correctement rendu dans le document.
- Vérifier la correspondance entre les identifiants des métadonnées, leurs valeurs et leur rendu Typst.
- Corriger les champs défectueux.

Critère d'acceptation : chaque métadonnée disponible est correctement insérée et rendue dans le document.

### T4.2 – Déplacer l'ajout d'image hors de la liste déroulante

- Retirer l'option d'ajout d'image de la liste déroulante des Quick Parts.
- Ajouter une petite icône dédiée à l'insertion d'image.
- Positionner cette icône à proximité des contrôles d'insertion des Quick Parts.
- Utiliser une icône visuellement identifiable (par exemple `image-plus`).

### T4.3 – Implémenter le sélecteur de fichier image

- Au clic sur l'icône d'ajout d'image, ouvrir le sélecteur de fichiers natif du système.
- Permettre de sélectionner une image depuis l'ordinateur.
- Vérifier les formats d'image pris en charge par Typst.
- Gérer proprement les erreurs de lecture et les formats non pris en charge.
- Insérer l'image dans l'en-tête ou le pied de page correspondant.

### T4.4 – Ajouter le réglage de hauteur de l'image

- Après insertion, permettre de définir la hauteur de l'image en pourcentage de la hauteur disponible de l'en-tête ou du pied de page.
- Ajouter un champ numérique ou un slider exprimé en pourcentage.
- Préserver les proportions de l'image.
- Vérifier que l'image ne déborde pas de la zone disponible.

Critère d'acceptation : l'utilisateur peut insérer une image et contrôler sa hauteur relativement à celle de l'en-tête ou du pied de page.

## 5. Onglet Typographie

### T5.1 – Ajouter des préréglages de guillemets

- Ajouter une liste déroulante de préréglages pour chacun des trois niveaux de guillemets.
- Proposer les principaux systèmes typographiques utilisés dans les langues européennes.
- Conserver la possibilité de saisir manuellement les caractères d'ouverture et de fermeture.
- Ne pas imposer les préréglages : ils doivent simplement faciliter la configuration.

Exemples de préréglages :

| Style    | Niveau 1 | Niveau 2 | Niveau 3 |
| -------- | -------- | -------- | -------- |
| Français | « »      | “ ”      | ‘ ’      |
| Anglais  | “ ”      | ‘ ’      | " "      |
| Allemand | „ “      | ‚ ‘      | « »      |
| Suisse   | « »      | ‹ ›      | “ ”      |

### T5.2 – Corriger l'aperçu des guillemets

- Examiner le composant d'aperçu actuel.
- Ajouter le troisième niveau de guillemets.
- Afficher un exemple d'imbrication réelle sur trois niveaux.
- Vérifier que les changements de configuration sont immédiatement visibles.

Critère d'acceptation : les trois niveaux configurés apparaissent dans l'aperçu, avec leur imbrication respective.

## 6. Onglet Couleurs et décors

### T6.1 – Rendre les pastilles de couleur interactives

- Examiner les pastilles de couleur présentes dans la section « Couleurs ».
- Corriger leur comportement au clic.
- Ouvrir le color picker natif ou celui déjà utilisé dans l'application.
- Permettre de sélectionner librement une couleur.
- Mettre à jour immédiatement la valeur et son aperçu.

### T6.2 – Uniformiser les contrôles de couleur des décors

- Reprendre le fonctionnement de la section « Couleurs ».
- Appliquer le même système aux champs de la section « Défauts du décor ».
- Présenter systématiquement :
  - Une pastille de couleur cliquable.
  - Un champ texte contenant le code hexadécimal.
- Permettre de modifier la couleur avec le color picker.
- Permettre de copier-coller directement un code hexadécimal.
- Synchroniser les deux contrôles dans les deux sens.
- Valider les codes hexadécimaux saisis et gérer les valeurs invalides.

Critère d'acceptation : toutes les couleurs du panneau utilisent une interface uniforme, avec color picker et saisie hexadécimale.

## 7. Onglet Bibliographie

### T7.1 – Vérifier la fidélité de l'aperçu des styles Zotero personnalisés

- Examiner le mécanisme de chargement des styles CSL personnalisés provenant de Zotero.
- Vérifier comment l'application interprète le fichier CSL.
- Comparer le résultat de l'aperçu avec le rendu attendu du style.
- Identifier les éventuelles différences d'interprétation.
- Vérifier notamment :
  - L'ordre des auteurs.
  - Les règles de ponctuation.
  - Les italiques.
  - Les titres et sous-titres.
  - Les dates.
  - Les séparateurs bibliographiques.
  - Les citations dans le texte et les références bibliographiques.
- Corriger les éventuelles divergences.

Critère d'acceptation : l'aperçu reflète fidèlement le style CSL personnalisé chargé.

Priorité secondaire

## 8. Cohérence générale de l'interface

### T8.1 – Uniformiser les composants de sélection de couleur

- Identifier tous les sélecteurs de couleur du panneau Normes du document.
- Utiliser un composant commun lorsque cela est techniquement pertinent.
- Garantir un comportement identique entre les différentes sections.
- Harmoniser les dimensions, espacements et interactions.

### T8.2 – Uniformiser les champs personnalisables

- Vérifier la cohérence visuelle des champs de saisie, listes déroulantes et sélecteurs.
- Harmoniser les interactions de recherche, autocomplétion et validation.
- Éviter de multiplier les implémentations spécifiques d'un même composant.

### T8.3 – Vérifier la persistance des paramètres

- Vérifier que chaque nouveau paramètre est correctement enregistré.
- Vérifier qu'il est restauré après fermeture et réouverture du document.
- Vérifier que les paramètres sont bien transmis au moteur de rendu Typst.
- Vérifier que les valeurs par défaut restent compatibles avec les anciens documents.

## 9. Tests et non-régression

### T9.1 – Tester chaque fonctionnalité individuellement

- Tester chaque tâche atomique.
- Vérifier les interactions entre les contrôles d'une même section.
- Vérifier les cas limites et les entrées invalides.

### T9.2 – Tester le rendu final du document

- Créer un document de test exploitant l'ensemble des nouvelles fonctionnalités.
- Générer le document Typst.
- Vérifier visuellement le résultat.
- Vérifier que les paramètres configurés dans le panneau correspondent au rendu réel.

### T9.3 – Vérifier la non-régression

- S'assurer que les fonctionnalités existantes du panneau Normes du document continuent de fonctionner.
- Vérifier la compatibilité avec les anciens documents.
- Éviter les modifications inutiles des autres panneaux de l'application.

---

## Compte rendu (2026-11-04)

Bilan final : **512 tests ordinaires + 118 épreuves** verts (épreuves lancées
une par une dans un HOME jetable). Portes : `cargo fmt --check`, `cargo
test --no-run`, `cargo clippy --all-targets -- -D warnings`, `cargo build`,
`git diff --check`, `cargo deny check` — toutes vertes.

### P0

- **T1.1 — polices système.** Cause : `Fonts::shared()` ne chargeait que les
  polices embarquées, `installed_font` un seul fichier par famille. Fichiers :
  `src/typst/fonts.rs`. Tests : parseur de `fc-list`, `familles_pour_choisir`.
- **T1.2 — saisie de police.** Cause : aucune suggestion. Fichiers :
  `src/ui/reglages.rs` (champ de recherche + 6 suggestions `ressemblantes`).
- **T4.1 — quick parts.** Cause : traduction codée en dur, `{document}` jamais
  alimenté, `compile()` sans entrées. Fichiers : `src/mise_en_page/gabarit.rs`,
  `src/typst/render.rs` (`nom_du_document`, `compile_avec_entrees`). Tests :
  `les_quick_parts_se_traduisent_toutes`, `les_quick_parts_rendent_leurs_valeurs`
  (vrai Typst CLI + pdftotext).
- **T6.1/T6.2 — couleurs.** Cause : pastille non cliquable, hex sans roue.
  Fichiers : `src/ui/reglages.rs` (`pastille` cliquable, `controle_couleur`
  unifié partout), `src/app/mod.rs` (`couleur_depuis`). Épreuve :
  `la_pastille_d_une_couleur_ecrit_sa_valeur`.

### P1

- **T2.1/T2.2/T2.3 — titres.** Colonne Couleur par niveau dans la grille,
  sections « Couleurs »/« Les défauts » supprimées, numérotation libre +
  aperçu (règle `numbering("1.1",1,2,3)` sondée au vrai Typst). Fichiers :
  `src/ui/reglages.rs`, `src/mise_en_page/modele.rs`, `src/ui/`.
- **T4.2/T4.3/T4.4 — images d'en-tête.** Icône dédiée, sélecteur natif `rfd`
  (png/jpg/gif/svg/webp, refus motivé), hauteur en % bornée 1–100. Test :
  `une_image_s_insere_et_se_regle`.
- **T5.1/T5.2 — guillemets.** Préréglages `GUILLEMETS_1/2/3`, saisie
  souveraine, aperçu imbriqué sur trois niveaux. Test :
  `l_apercu_des_guillemets_imbrique_trois_niveaux`.

### P2

- **T3.1 — formats de dates documentés.** Bloc de jetons + exemples sous
  « Format des dates pleines » (`src/ui/reglages.rs`).
- **T3.2 — langue des dates.** Cause : noms codés en dur en français.
  Fichiers : `src/langage/dates.rs` (`Regles.langue`, `nom_du_mois`,
  `nom_du_jour`, `jour_de_la_semaine`, jetons `MMMM MMM dddd ddd` — tableau
  des jetons remis du plus long au plus court), `src/mise_en_page/modele.rs`
  (`Champ::DatesLangue`), `src/ui/reglages.rs` (sélecteur français/anglais/
  allemand/italien). Test : `les_noms_se_localisent_sans_toucher_au_format`
  (le 14 juillet 1789 est un mardi — vérifié par l'algorithme).
- **T7.1 — fidélité de l'aperçu CSL.** L'aperçu compile avec le **même
  moteur** et le **même style** que le document (fidélité structurelle) ;
  divergence trouvée et corrigée : l'extraction du cadre perdait gras et
  italiques — `extraire_texte` les marque maintenant (`*…*`, `_…_`) via la
  variante de police. Échantillon enrichi (trois auteurs pour l'ordre et
  « et al. », sous-titre, mois). Fichier : `src/typst/styles.rs`. Tests :
  `l_apercu_montre_le_gras_et_l_italique`, `l_apercu_est_rendu_par_le_moteur`.

### P3

- **T8.1 — sélecteurs de couleur uniformisés.** Tout passe par
  `pastille()`/`controle_couleur()` (Couleurs, décor, niveaux de titres) :
  même geste, même roue ouverte sur la couleur courante. La grille des titres
  garde la seule pastille (largeur de colonne) — même interaction.
- **T8.2 — champs uniformisés.** Coque canonique `champ_texte` (même police,
  marge, taille) partout — y compris l'éditeur de quick parts ; listes
  déroulantes à largeur 170 commune (`choix`, `choix_langue`,
  `choix_defaut_vide`, `choix_police` avec recherche).
- **T8.3 — persistance.** Tests : `les_reglages_neufs_se_persistent_et_les_anciens_chargent`
  (aller-retour toml + ancien fichier aux défauts), `les_styles_des_titres_se_transmettent`
  (gras/italique/souligné au gabarit).
- **T9.1 — tests individuels.** Chaque tâche a son test ou son épreuve ;
  épreuve d'interaction par section : `une_section_se_regle_dans_son_entier`
  (niveaux indépendants, mesure nue → em, négatif refusé, langue en code).
- **T9.2 — rendu final.** Test `le_document_de_synthese_se_compose_pour_de_vrai` :
  un document exploitant toutes les nouveautés, composé au vrai Typst avec
  `--input auteur=Alice`, relu par pdftotext. **La vérification visuelle
  reste à l'auteur** (règle établie : aucune capture d'écran par l'agent).
- **T9.3 — non-régression.** 512 tests + 118 épreuves verts ; compatibilité
  des anciens documents prouvée (champs `serde` par défaut) ; les autres
  panneaux n'ont pas été touchés.

### Deux courses parallèles trouvées et réglées (hors périmètre initial)

- `vers_la_corbeille` : la prise de nom était check-then-rename — deux courses
  choisissaient le même nom et la seconde écrasait la première. Prise
  atomique (fichier ou dossier selon la source).
- L'atelier d'import n'était clavé que par PID : deux imports parallèles
  s'effaçaient les fichiers stagés (« No such file or directory »). Clé
  unique par import (nanosecondes).

### Restes connus

- Acceptation visuelle par l'auteur (rendu réel, vérifications sur
  l'application réelle) — hors de portée de l'agent par règle établie.
- Windows/macOS : hors de cette machine.
- Projets hérités sans `document.tw` hors du dossier des projets : non
  reconnus par `verifier_destination_export` (limite notée en bug-report-7).