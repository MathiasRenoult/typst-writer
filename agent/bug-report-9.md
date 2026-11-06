Backlog – Corrections et évolutions
1. Panneau Normes du document
1.1 – Bloquer les interactions avec l'éditeur en arrière-plan
- Lorsque le panneau Normes est ouvert, neutraliser toutes les interactions avec le reste de l'interface.
- Empêcher notamment les clics sur les swatches de couleur de modifier directement le contenu du document.
- Les interactions doivent être limitées au panneau actif.
1.2 – Corriger l'application des styles de titres
Actuellement, seules la taille et la casse des titres sont correctement appliquées.
- Corriger l'application du gras, de l'italique et du soulignement.
- Corriger l'application des couleurs personnalisées.
- S'assurer que toutes les propriétés configurées dans les normes correspondent au rendu final.
1.3 – Améliorer le sélecteur de polices
- Conserver la recherche de polices existante.
- Déplacer les résultats de recherche dans un véritable menu déroulant (dropdown).
- Éviter que les résultats apparaissent directement sous la barre de recherche.
- Permettre également de parcourir la liste complète des polices disponibles.
1.4 – Corriger l'application des polices
- Actuellement, changer la police dans les normes ne semble pas modifier le rendu du document.
- Corriger la transmission et l'application de la police sélectionnée au moteur Typst.
2. Barre supérieure et zoom
2.1 – Déplacer les contrôles de zoom dans la Top Bar
Déplacer les quatre contrôles suivants dans la barre supérieure et les centrer :
- Réduire le zoom.
- Ajuster à la largeur.
- Ajuster à la hauteur.
- Augmenter le zoom.
2.2 – Supprimer les anciens contrôles de zoom
- Supprimer les boutons de zoom actuellement présents dans la fenêtre de rendu.
- Éviter toute duplication des contrôles.
3. Glossaire
3.1 – État actuel
- L'implémentation actuelle est satisfaisante pour une première version.
- Pas de modification demandée pour le moment.
4. Refonte du moteur de rendu
Projet majeur – Priorité stratégique
4.1 – Remplacer le système de rendu vectoriel actuel par un lecteur PDF
Actuellement, l'application semble récupérer les vecteurs produits par le moteur de composition Typst, puis les dessiner elle-même.

L'objectif est de remplacer cette approche par un système basé sur le PDF :
- Faire générer directement un PDF par Typst.
- Intégrer un véritable lecteur PDF dans l'application.
- Afficher le PDF directement dans la fenêtre de rendu.
- Abandonner progressivement le système actuel de récupération et de dessin manuel des vecteurs.
4.2 – Garantir un zoom de qualité
- Préserver la qualité vectorielle du PDF lors du zoom.
- Éviter la pixellisation actuellement observable lors des forts niveaux de zoom.
- Permettre un zoom fluide, avec un rendu net quelle que soit l'échelle.
4.3 – Exploiter les fonctionnalités natives du lecteur PDF
Privilégier un lecteur PDF existant plutôt que de réimplémenter ses fonctionnalités.

L'objectif est notamment de bénéficier de :
- La navigation entre les pages.
- L'affichage vectoriel.
- La gestion du zoom.
- Un rendu typographique propre.
- Une bonne gestion des performances.
---

# Compte rendu de traitement (2026-11-05)

Toutes les tâches d'interface sont traitées et vérifiées ; la refonte
PDF est **proposée, non implémentée** (4.1-4.3 exigent une architecture
présentée avant tout code). Détails complets dans
`agent/Pré-Beta/suivi-developpement-beta.md` ; proposition dans
`agent/proposition-lecteur-pdf.md`.

## 1.1 – Modalité du panneau Normes : FAIT
Cause : l'éditeur gardait le focus à l'ouverture du panneau. Modifié :
`src/app/reglages.rs` (`lose_focus`). Test `les_normes_prennent_les_mains`,
**prouvé par mutation**. Reste : rien.

## 1.2 – Styles de titres au rendu : FAIT (P0)
Causes, deux mécanismes prouvés au vrai moteur (appliquer → assembler →
composer) : (a) le gras ne pouvait pas se **désactiver** (le gras par
défaut des titres survécait à la case décochée — `weight: "regular"`
maintenant émis) ; (b) la typographie des normes perdait contre le
`#set text` de l'assemblage (règle déplacée sur les trames : `show text:
set text`), et les réglages de titre n'avaient qu'une place qui l'emporte
(règle intérieure). **Suite trouvée en cours de route** : la langue du
moteur (sommaire, bibliographie) se lit à la portée d'élément — les
deux courses d'acceptation l'ont vu en premier (« contents »,
« bibliography ») et la typographie s'écrit donc en **double portée**
(`set text` + `show text: set text`). Modifié : `src/mise_en_page/gabarit.rs`
; test permanent `les_normes_se_rendent_au_moteur` (2 tests) dans
`src/langage/pipeline.rs` (la sonde est devenue le test). Reste :
l'acceptation visuelle, à l'auteur.

## 1.3 – Sélecteur de polices : FAIT (P1)
Le choix est un **vrai menu déroulant rechercheable**
(`iced::widget::combo_box` : liste complète parcourable, recherche dont
le filtre ignore casse/espaces/signes, saisie directe conservée,
aperçu gardé). L'ancienne liste de suggestions en ligne et ses helpers
(`familles_ressemblant`, `ressemblantes`, leurs tests) partent avec lui.
Modifié : `src/ui/reglages.rs`, `src/app/state.rs`, `src/app/mod.rs`
(`police_combo`), `src/typst/fonts.rs`. Reste : l'aspect du menu dans
l'app, à l'auteur.

## 1.4 – Application de la police : FAIT (P0)
Cause : le `#set text(font: …, size: …)` du gabarit d'origine, placé
après dans le fil, l'emportait sur les normes — la police ne changeait
jamais. Corrigé par la portée de trame (1.2), prouvé par
`la_police_des_normes_l_emporte_sur_l_assemblage` (**mutation** :
correctif retiré → rouge). Transmission par le champ : chemin générique
éprouvé (T8.3). Reste : rien.

## 2.1 + 2.2 – Zoom dans la barre supérieure : FAIT
Les quatre commandes (« Réduire le zoom », « Ajuster à la largeur »,
« Ajuster à la hauteur », « Augmenter le zoom ») et leur pourcentage
vivent dans le **segment central** de la barre supérieure, en mode Rendu.
La barre de zoom de la fenêtre de rendu (« − », « + », « W », « H ») est
supprimée : aucune duplication. Modifié : `src/ui/toolbar.rs`,
`src/ui/preview.rs`. Reste : l'emplacement à l'écran, à l'auteur.

## 3 – Glossaire : inchangé (comme demandé).

## 4 – Refonte du moteur de rendu : FAITE, puis fluidifiée

Étude d'abord (`agent/proposition-lecteur-pdf.md`), implémentation
ensuite sur la décision de l'auteur : **pdf_render 1.3 (Apache-2.0
OU MIT) est LE moteur de rendu** — il interprète le PDF exporté (le
fichier livré est la source d'affichage) et rasterise ; `typst-render`
est retiré. MuPDF (AGPL) et Poppler (GPL) étaient écartés sur licence,
pdfium-render reste noté pour plus tard.

4.1/4.2/4.3 — et la plainte de fluidité venue ensuite : le zoom était
« lent ». Mesuré en release : une page entière à 10 px/pt coûtait
**619 ms**, la molette en lançant jusqu'à quatre par cran. Le moteur
est vendu (`vendor/pdf-render-1.3.0`, `PATCHES.md`, un champ `origin`
neutre) et la **fenêtre visible seule** est rasterisée : **24 ms à tout
zoom** (27 fois plus vite au zoom maximal), le geste de zoom ne
déclenche qu'un rendu net à sa fin, et défiler dans la marge est
gratuit. Preuves : identité région = recadrage du rendu plein,
épreuve du geste — les deux prouvées par mutation — et le banc de
mesure (`sonde_fluidite`) rejouable.

Les fonctions de lecteur restantes (recherche, sélection, liens,
tâche 4.3) se câbleront sur les API de texte du moteur au fil des
besoins — l'architecture les porte.
## Vérification finale

**532 tests ordinaires** + la batterie canonique des épreuves (un
HOME jetable chacune) ; six portes vertes : `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo build`, `cargo test`,
`git diff --check`, `cargo deny check`. Les épreuves nouvelles sont
prouvées par mutation. Aucun commit, aucun push.
