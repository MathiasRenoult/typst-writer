# Bug report 04 — Interface, éditeur et normes typographiques

> **Nature :** rapport de bugs et demandes d'amélioration issu d'une nouvelle session de test.
> **Périmètre :** panneau latéral gauche, barre supérieure, éditeur, éditeur de thèmes et fenêtre des normes.
> **Important :** ce rapport est distinct du cahier des charges général précédent. Certaines demandes se recoupent, mais les observations ci-dessous décrivent l'état constaté lors de cette nouvelle session. Il faut vérifier le comportement réel dans le code et l'application, sans supposer que les demandes similaires sont déjà satisfaites.

## Consignes générales à l'agent

1. Inspecter l'implémentation actuelle avant toute modification. Reproduire chaque bug lorsque c'est possible et identifier sa cause.
2. Comparer ce rapport avec les demandes et corrections déjà traitées dans le dépôt. Réutiliser les solutions existantes et éviter les doublons.
3. Ne pas supposer qu'une demande similaire dans un rapport précédent est déjà satisfaite : vérifier le comportement réel.
4. Regrouper les corrections par composants et dépendances. Éviter les modifications massives mélangeant des problèmes sans rapport.
5. Préserver les données et les comportements existants. Toute opération de sauvegarde, modification de thème ou de norme doit rester sûre en cas d'erreur.
6. Ajouter ou adapter des tests lorsque pertinent. Pour les problèmes d'interface, vérifier le comportement réel, pas seulement la compilation.
7. Mettre à jour `IMPLEMENTATION_PROGRESS.md` avec les identifiants ci-dessous, leur état (`TODO`, `IN_PROGRESS`, `DONE`, `BLOCKED`), les fichiers concernés, les tests effectués et les éventuelles réserves.
8. Ne déclarer une demande `DONE` qu'après vérification. Si un comportement ne peut pas être testé automatiquement, préciser le contrôle manuel nécessaire.
9. Ne pas faire de push distant.

### Priorités proposées

- **P0 — Intégrité ou perte de données :** aucun problème de cette catégorie n'est explicitement signalé dans ce rapport.
- **P1 — Bugs fonctionnels importants :** interaction impossible, contenu masqué ou comportement d'édition incorrect.
- **P2 — Ergonomie et cohérence de l'interface.**
- **P3 — Évolution fonctionnelle.**

Les priorités sont indicatives et peuvent être ajustées si le diagnostic révèle un risque plus important.

---

# 1. Panneau latéral gauche

## LEFT-04-01 — Rendre le panneau latéral réouvrable après sa fermeture
**Priorité : P1 — Interaction bloquante**

### Problème observé
L'icône permettant de fermer le panneau latéral se trouve peut-être à un emplacement acceptable. Cependant, une fois le panneau fermé, aucun moyen évident ne permet de le rouvrir.

### Comportement attendu
- Le panneau doit pouvoir être ouvert et fermé à tout moment.
- Lorsqu'il est fermé, un contrôle de réouverture doit rester accessible et visible.
- Le contrôle d'ouverture/fermeture doit être placé à un emplacement cohérent, éventuellement en dehors du panneau lui-même.
- L'état ouvert/fermé doit être compréhensible visuellement et le contrôle utilisable au clavier.

### Vérifications
Tester les ouvertures et fermetures répétées, le redimensionnement de la fenêtre et les différentes dispositions de panneaux. Aucun état ne doit laisser le panneau inaccessible.

## LEFT-04-02 — Supprimer l'onglet « Documents »
**Priorité : P2**

L'onglet « Documents » affiche principalement le nom technique `document.tw` et le nombre de mots. Ces informations ne sont pas utiles dans ce panneau.

- Supprimer l'onglet « Documents » du panneau latéral.
- Vérifier qu'il ne contient pas une commande utile qui serait autrement inaccessible.
- Ne pas supprimer la gestion interne des documents ni les métadonnées nécessaires au fonctionnement de l'application.

## LEFT-04-03 — Déplacer l'onglet portant le nom du document
**Priorité : P2**

Le premier onglet, qui porte le nom du document, est mal placé dans le panneau latéral.

- Retirer cet onglet de la liste des onglets du panneau.
- Présenter le nom du document dans une zone supérieure plus pertinente, idéalement au même niveau que le contrôle d'ouverture/fermeture du panneau ou à proximité.
- Éviter de dupliquer une information déjà clairement affichée dans les onglets de document.
- Conserver une hiérarchie visuelle claire entre le nom du document, les commandes du panneau et les onglets fonctionnels.

## LEFT-04-04 — Corriger l'alignement vertical des projets
**Priorité : P2**

Dans l'onglet « Projets », le nom de chaque projet est aligné en haut à gauche de sa boîte au lieu d'être centré verticalement et aligné à gauche.

- Centrer verticalement le nom du projet dans son conteneur.
- Conserver un alignement horizontal à gauche.
- Appliquer une disposition cohérente lorsque le nom est long ou que des boutons d'action sont présents.

## LEFT-04-05 — Corriger l'alignement du bouton « Nouveau projet »
**Priorité : P2**

Le contenu du bouton « Nouveau projet » n'est pas correctement aligné à l'intérieur du bouton.

- Centrer verticalement et horizontalement les éléments du bouton selon sa composition (icône et libellé).
- Corriger l'alignement interne sans modifier inutilement sa taille ou sa position globale.
- Vérifier le rendu à différentes tailles de fenêtre.

---

# 2. Barre supérieure (Top bar)

## TOPBAR-04-01 — Réorganiser les groupes de commandes et leur alignement
**Priorité : P2**

### Problème observé
Les icônes sont regroupées vers la gauche. La barre supérieure manque de répartition et de hiérarchie.

### Comportement attendu
Organiser la barre supérieure en groupes fonctionnels distincts :
- commandes alignées à gauche ;
- commandes centrées dans la barre ;
- commandes alignées à droite.

La répartition exacte doit découler de la fonction des commandes et de la structure de l'application. Utiliser des groupes cohérents, des espacements réguliers et des séparations visuelles discrètes si nécessaire. Éviter les chevauchements lorsque la fenêtre est étroite.

## TOPBAR-04-02 — Supprimer les commandes de gestion de fichiers devenues inutiles
**Priorité : P2**

Les commandes suivantes sont inutiles dans le flux de travail actuel :
- ouvrir un document ;
- importer un document ;
- enregistrer ;
- enregistrer sous.

Retirer ces boutons de la barre supérieure. Avant suppression, vérifier que leurs fonctions restent accessibles par un autre mécanisme lorsque nécessaire. Ne pas supprimer les fonctions internes d'ouverture, d'import ou de sauvegarde, ni affaiblir la sauvegarde automatique ou la persistance des documents.

## TOPBAR-04-03 — Centrer correctement les icônes des commandes de mise en forme
**Priorité : P2**

Plusieurs icônes de mise en forme sont positionnées en haut à gauche de leur bouton au lieu d'être centrées. Le problème concerne notamment le gras, l'italique, les listes à puces et les commandes similaires.

- Centrer horizontalement et verticalement chaque icône dans la zone de son bouton.
- Vérifier les dimensions des boutons, les marges internes et l'alignement des conteneurs.
- Harmoniser les tailles et les alignements sur l'ensemble du groupe.
- Vérifier le rendu à différentes tailles d'écran et avec les différents états des boutons.

## TOPBAR-04-04 — Corriger le menu de sélection des niveaux de titre
**Priorité : P1 — Commande actuellement inutilisable**

### Problème observé
Le bouton d'insertion de titre ouvre un panneau proposant les niveaux H1 à H6. Cependant, dès que l'utilisateur déplace le pointeur du bouton vers le panneau, celui-ci disparaît. Il est donc impossible ou très difficile de sélectionner un niveau.

### Comportement attendu
- Le panneau doit rester ouvert lorsque le pointeur passe du bouton déclencheur vers son contenu.
- Il doit se fermer lorsque l'utilisateur clique à l'extérieur, sélectionne un niveau ou utilise une commande de fermeture prévue.
- Le menu doit rester utilisable au clavier.
- Vérifier la gestion des zones de survol, du focus et de la propagation des événements.

## TOPBAR-04-05 — Retirer le choix de police de la barre supérieure
**Priorité : P2**

Un contrôle de sélection de police est présent dans la barre supérieure. Ce réglage relève des normes du document et n'a pas sa place parmi les commandes rapides de l'éditeur.

- Retirer le contrôle de police de la barre supérieure.
- Gérer les choix de police dans la fenêtre des normes du document.
- Vérifier que le réglage de police reste disponible et fonctionnel dans les normes.
- Ne pas supprimer la prise en charge des polices dans le moteur de rendu.

## TOPBAR-04-06 — Rendre visible la dernière couleur sélectionnée
**Priorité : P1**

Le bouton qui applique la dernière couleur choisie est transparent et ne permet pas de connaître la couleur qu'il appliquera.

- Afficher une pastille ou un indicateur représentant réellement la dernière couleur sélectionnée.
- Faire correspondre exactement la couleur affichée à celle qui sera appliquée au texte.
- Garder l'indicateur lisible avec des couleurs très claires, très foncées ou proches de la couleur de fond.

## TOPBAR-04-07 — Synchroniser le sélecteur de couleur et la couleur courante
**Priorité : P1**

Le sélecteur de couleur situé à droite du bouton d'application fonctionne, mais choisir une nouvelle couleur ne met pas à jour la dernière couleur sélectionnée.

- Toute sélection validée dans le color picker doit devenir la couleur courante.
- Le bouton d'application doit immédiatement afficher cette nouvelle couleur.
- Un clic sur le bouton d'application doit utiliser exactement cette couleur.
- L'état doit rester cohérent après fermeture et réouverture du sélecteur, selon la portée prévue par l'application.

## TOPBAR-04-08 — Améliorer le color picker de l'éditeur
**Priorité : P2**

Le sélecteur doit proposer à la fois des couleurs prédéfinies et la liberté de choisir une couleur personnalisée.

- Fournir une palette de couleurs de base.
- Permettre de choisir librement une couleur avec un contrôle adapté.
- Conserver la possibilité de saisir une valeur précise si elle existe déjà dans le système.
- Mettre à jour la couleur courante après validation (voir TOPBAR-04-07).
- Présenter clairement la couleur sélectionnée et permettre de confirmer ou d'annuler le choix.

Ne pas confondre ce sélecteur avec celui de l'éditeur de thèmes : ils peuvent partager un composant technique, mais leurs états et leurs effets doivent rester distincts.

## TOPBAR-04-09 — Supprimer les commandes d'aperçu inutiles
**Priorité : P2**

Supprimer de la barre supérieure :
- le bouton « Masquer l'aperçu » ;
- le bouton qui fait suivre à l'aperçu le chapitre actuellement écrit.

Vérifier que leur suppression ne désactive pas une fonction nécessaire à l'affichage du rendu ou à la navigation. Ne pas supprimer la fenêtre de rendu elle-même.

## TOPBAR-04-10 — Replacer la commande des normes
**Priorité : P2**

Le bouton permettant d'accéder aux normes du document devrait être déplacé à un autre emplacement, éventuellement tout à gauche de la barre supérieure.

Choisir son emplacement en fonction de la nouvelle organisation globale de la barre. Le placer dans un groupe identifiable et éviter de le mélanger aux commandes de mise en forme du texte.

---

# 3. Éditeur de thèmes

## THEME-EDITOR-04-01 — Intégrer le color picker dans l'application
**Priorité : P1 — Incompatibilité avec le gestionnaire de fenêtres**

### Problème observé
L'éditeur de thèmes fonctionne mieux qu'auparavant. Cependant, son color picker ouvre une fenêtre séparée. Dans un environnement utilisant un compositeur de fenêtres tel que Hyprland, cette fenêtre séparée entraîne des problèmes et rend le sélecteur difficilement exploitable.

### Comportement attendu
- Ouvrir le color picker sous la forme d'un panneau, popover ou overlay intégré à la fenêtre de l'application.
- Ne pas créer une fenêtre native séparée pour cette interaction.
- Le sélecteur doit rester attaché visuellement au champ ou au bouton qui l'a ouvert.
- Permettre de choisir une couleur, de valider ou d'annuler, puis de revenir naturellement à l'éditeur de thèmes.
- Gérer correctement le focus, le clic extérieur, le redimensionnement et les limites de la fenêtre.
- Vérifier le comportement sous Hyprland ainsi que dans les autres environnements pris en charge.

Privilégier un composant de color picker intégré à l'interface iced plutôt qu'un dialogue système externe. Réutiliser un composant existant si cela ne crée pas de dépendance ou de complexité injustifiée.

## THEME-EDITOR-04-02 — Supprimer les pastilles blanches et noires inutiles
**Priorité : P2**

Des pastilles blanches et noires apparaissent à l'extrémité droite du panneau de l'éditeur de thèmes. Elles n'apportent pas d'information ou de fonction utile dans le flux de travail actuel.

- Identifier leur fonction dans le code.
- Si elles ne servent qu'à une prévisualisation ou à une action non nécessaire, les supprimer.
- Si elles sont liées à une fonction utile, vérifier que celle-ci reste accessible ailleurs avant de les retirer.
- Ne pas supprimer les véritables champs de couleur ou les contrôles nécessaires à l'édition du thème.

---

# 4. Éditeur de texte

## EDITOR-04-01 — Supprimer intégralement un glyphe de note de bas de page ou de référence
**Priorité : P1 — Intégrité de l'édition**

### Problème observé
Certains éléments de note de bas de page ou de référence sont représentés dans l'éditeur par un glyphe. La clé de la note ou de la référence est masquée derrière ce glyphe.

Lorsqu'on place le curseur juste devant le glyphe et qu'on appuie sur `Backspace`, le comportement est parfois incorrect : au lieu de supprimer l'élément complet, le glyphe disparaît ou se transforme et laisse apparaître les caractères de la clé qui se trouvaient derrière.

### Comportement attendu
- Un appui sur `Backspace` juste après un glyphe de note ou de référence doit supprimer l'élément complet en une seule opération.
- La clé interne ne doit jamais être révélée comme du texte ordinaire à la suite d'une suppression.
- Le comportement doit être cohérent pour les notes de bas de page simples et les références bibliographiques.
- Vérifier également le comportement de `Delete` lorsque le curseur se trouve juste avant l'élément, si cette opération est prise en charge.
- Une suppression doit laisser le document dans un état syntaxiquement valide et ne pas corrompre les éléments adjacents.

### Analyse technique attendue
Identifier comment ces glyphes sont représentés dans le modèle de texte, comment les positions du curseur sont converties et comment les suppressions sont appliquées. Corriger la gestion des plages ou des éléments atomiques à la source plutôt que masquer les caractères révélés après coup.

### Tests
Couvrir au minimum :
- curseur immédiatement après le glyphe puis `Backspace` ;
- curseur immédiatement avant le glyphe puis `Delete` ;
- suppression au milieu d'un paragraphe ;
- éléments adjacents au texte, à une ponctuation ou à un autre glyphe ;
- annulation/rétablissement de la suppression si l'éditeur prend en charge undo/redo.

## EDITOR-04-02 — Fermer automatiquement l'éditeur flottant d'une note de bas de page
**Priorité : P1**

### Problème observé
Cliquer sur une note de bas de page simple ouvre une pop-up ou un overlay permettant de la modifier. Pour le fermer, il faut actuellement utiliser la croix située en bas.

### Comportement attendu
- Lorsque l'utilisateur clique dans l'éditeur ailleurs que dans l'overlay, celui-ci se ferme automatiquement.
- Un clic à l'intérieur de l'overlay ne doit pas le fermer.
- Les interactions avec les contrôles internes de l'overlay doivent rester possibles.
- La croix de fermeture peut être conservée comme moyen de fermeture explicite.
- Ne pas fermer l'overlay pendant que l'utilisateur saisit ou sélectionne du texte à l'intérieur.

Tester un clic dans le texte, sur une autre note, sur une référence, dans le rendu et dans l'overlay lui-même. Définir le comportement attendu pour un clic sur une zone non interactive de l'application.

## EDITOR-04-03 — Corriger le rendu graphique incomplet des dernières lignes
**Priorité : P1 — Contenu visuellement masqué**

### Problème observé
Il arrive que les dernières lignes du contenu de l'éditeur ne soient pas rendues graphiquement. Le texte existe bien : lorsque le curseur progresse dans la zone concernée, les lignes et les caractères apparaissent progressivement.

### Comportement attendu
- Tout le texte présent dans le document doit être rendu immédiatement et intégralement, sans devoir déplacer le curseur pour le faire apparaître.
- Le rendu doit rester correct après saisie, suppression, collage, défilement, redimensionnement et changement de fenêtre.
- Les dernières lignes ne doivent pas être coupées par une mauvaise estimation de la hauteur du viewport ou du contenu.
- Ne pas corriger le problème par un rafraîchissement forcé permanent qui dégraderait les performances.

### Diagnostic attendu
Examiner notamment :
- le calcul de la hauteur du contenu et de la zone visible ;
- la virtualisation ou le rendu partiel des lignes, si utilisé ;
- l'invalidation et la mise à jour des zones graphiques ;
- le clipping, les coordonnées et les limites des widgets ;
- le déclenchement du rendu lors des changements de curseur et de focus.

Identifier la cause exacte avant de choisir une correction.

### Tests
Reproduire avec des documents courts et longs, en particulier lorsque le curseur est loin de la fin du document. Vérifier que toutes les lignes restent visibles sans interaction supplémentaire.

---

# 5. Fenêtre des normes

## NORMS-04-01 — Retirer les flèches haut/bas des champs qui ne sont pas numériques
**Priorité : P2**

De nombreux champs de la fenêtre des normes affichent des flèches haut et bas alors que les valeurs concernées ne sont pas des valeurs numériques à incrémenter.

- Réserver les contrôles d'incrémentation aux valeurs numériques pour lesquelles cette interaction est pertinente.
- Lorsqu'un choix doit être fait parmi un ensemble fini d'options, utiliser une liste déroulante ou un contrôle de sélection adapté.
- Pour les champs textuels ou les valeurs qui ne se prêtent pas à une incrémentation, ne pas afficher de flèches.
- Vérifier que le contrôle utilisé correspond au type de donnée et au comportement attendu.
- Ne pas modifier les valeurs ou les formats stockés uniquement pour adapter l'apparence des contrôles.

## NORMS-04-02 — Configurer individuellement l'apparence de chaque niveau de titre
**Priorité : P2 — Évolution fonctionnelle majeure**

### Problème observé
Il n'existe pas de moyen suffisamment complet pour définir séparément l'apparence de chaque niveau de titre. Chaque niveau doit pouvoir avoir son propre style.

### Comportement attendu
Pour chacun des six niveaux (`Titre 1` à `Titre 6`, ou `H1` à `H6` selon la terminologie de l'interface), permettre de configurer individuellement :

- **Couleur** du texte ;
- **Casse** (normale, majuscules, petites capitales, etc., selon les fonctions prises en charge) ;
- **Graisse / poids** (normal, semi-gras, gras, etc., selon les possibilités du moteur) ;
- **Style** (italique et autres styles déjà pris en charge) ;
- **Taille** ;
- **Espacement** avant et après le titre ;
- **Autres propriétés typographiques pertinentes** déjà supportées par le moteur de rendu.

Les six niveaux doivent pouvoir avoir des configurations différentes. Une modification du style de H2 ne doit pas modifier automatiquement H1 ou H3, sauf si l'utilisateur choisit explicitement une fonction de copie ou d'héritage.

### Présentation souhaitée
- Proposer une sélection claire du niveau à configurer.
- Afficher les propriétés de ce niveau dans un panneau organisé.
- Fournir un aperçu du résultat.
- Permettre de passer d'un niveau à l'autre sans perdre les modifications non enregistrées.
- Enregistrer ces paramètres dans la norme active et les appliquer effectivement au rendu.

### Numérotation
La numérotation des titres a été évoquée puis laissée en suspens. **Ne pas implémenter une configuration de numérotation dans le cadre de cette demande sans vérifier si elle existe déjà et sans définir précisément le besoin.** La priorité est la personnalisation typographique indépendante des six niveaux.

## NORMS-04-03 — Déplacer la gestion des modèles de normes
**Priorité : P2**

### Problème observé
La section permettant de charger un modèle de normes, de le modifier et d'enregistrer un nouveau modèle est située au milieu de l'onglet « Mise en page ». Elle est difficile à repérer et paraît mal placée.

### Comportement attendu
- Déplacer cette section tout en haut de la fenêtre des normes, dans une zone clairement identifiable ; ou
- créer un onglet ou une section dédiée à la gestion des modèles si cela produit une organisation plus claire.

Examiner la structure actuelle et choisir la solution la plus cohérente. Ne pas conserver la section au milieu des paramètres de mise en page.

### Fonctions à préserver
- charger un modèle ;
- modifier les paramètres du modèle ;
- enregistrer un nouveau modèle.

Vérifier que le déplacement de l'interface ne modifie pas le comportement de chargement ou d'enregistrement des modèles.

---

# 6. Dépendances et regroupements

| Groupe | Identifiants | Dépendance / justification |
|---|---|---|
| Architecture du panneau latéral | LEFT-04-01 à LEFT-04-05 | Revoir ensemble l'en-tête, les contrôles d'ouverture et la disposition des onglets et projets. |
| Organisation de la barre supérieure | TOPBAR-04-01 à TOPBAR-04-05, TOPBAR-04-09, TOPBAR-04-10 | Définir les groupes et leur emplacement avant de peaufiner les contrôles individuels. |
| Couleur de l'éditeur | TOPBAR-04-06 à TOPBAR-04-08 | Le choix, l'affichage et l'application de la couleur doivent partager un état cohérent. |
| Color picker de thèmes | THEME-EDITOR-04-01, THEME-EDITOR-04-02 | Revoir le composant intégré et supprimer les contrôles devenus inutiles. |
| Éléments atomiques de l'éditeur | EDITOR-04-01 | Nécessite d'examiner le modèle de texte, les glyphes, le curseur et les opérations de suppression. |
| Overlay de note | EDITOR-04-02 | Nécessite une gestion correcte du focus et des clics extérieurs. |
| Rendu des lignes | EDITOR-04-03 | À diagnostiquer indépendamment des problèmes de contenu ou de syntaxe. |
| Configuration des normes | NORMS-04-01 à NORMS-04-03 | Revoir les contrôles et l'organisation de la fenêtre, puis enrichir la configuration des titres. |

# 7. Ordre de traitement recommandé

1. **Bugs qui empêchent une interaction ou masquent du contenu :**
   - LEFT-04-01 ;
   - TOPBAR-04-04 ;
   - EDITOR-04-01 ;
   - EDITOR-04-02 ;
   - EDITOR-04-03.

2. **Cohérence des commandes et de la sélection de couleur :**
   - TOPBAR-04-06 ;
   - TOPBAR-04-07 ;
   - TOPBAR-04-08 ;
   - THEME-EDITOR-04-01.

3. **Réorganisation des panneaux et de la barre supérieure :**
   - LEFT-04-02 à LEFT-04-05 ;
   - TOPBAR-04-01 à TOPBAR-04-05 ;
   - TOPBAR-04-09 ;
   - TOPBAR-04-10.

4. **Améliorations de la fenêtre des normes :**
   - NORMS-04-01 ;
   - NORMS-04-03 ;
   - NORMS-04-02.

5. **Finitions de l'éditeur de thèmes :**
   - THEME-EDITOR-04-02.

Cet ordre peut être adapté si le diagnostic révèle des dépendances techniques supplémentaires.

# 8. Définition de l'achèvement

Le rapport est traité lorsque :
- chaque identifiant possède un état documenté dans `IMPLEMENTATION_PROGRESS.md` ;
- les bugs fonctionnels ont été reproduits puis vérifiés après correction, ou leur impossibilité de reproduction est explicitement documentée ;
- les changements d'interface sont cohérents avec le reste de l'application ;
- les paramètres de normes sont réellement persistés et appliqués au rendu ;
- les tests automatisés pertinents passent ;
- les scénarios manuels nécessaires sont consignés ;
- le projet compile sans nouvelle erreur ;
- aucune régression connue n'a été introduite.

**Ne pas se limiter à modifier l'apparence des composants : corriger les causes des bugs, en particulier pour les glyphes de référence et le rendu incomplet des dernières lignes.**
