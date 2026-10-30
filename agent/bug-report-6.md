# Bug Report 6 — Navigation, alignement et éditeur de thème

## Consignes générales

- Examiner l'implémentation actuelle avant toute modification et reproduire les problèmes lorsque possible.
- Identifier les causes réelles plutôt que corriger uniquement les symptômes.
- Comparer ce rapport aux rapports précédents (`observations_application_revisees.md`, `bug-report-4.md` et `bug-report-5.md`). Vérifier l'état réel du code avant de considérer un point comme déjà corrigé.
- Préserver les fonctionnalités existantes et vérifier les interactions entre onglets, panneaux, top bar et éditeur de thème.
- Pour chaque tâche, consigner l'état, les fichiers concernés, les décisions et les tests dans `IMPLEMENTATION_PROGRESS.md`.
- Ne pas inventer de comportement lorsque la demande est ambiguë : documenter et marquer `BLOCKED` si une décision humaine est indispensable.

## 1. Navigation entre les projets et les onglets

### NAV-06-01 — Ouvrir un projet dans un nouvel onglet avec Shift-clic ou Ctrl-clic

**Priorité : P1**

Actuellement, `Shift + clic` sur un projet dans le panneau de gauche n'ouvre pas le projet dans un nouvel onglet comme attendu.

Comportement attendu :
- Un clic simple sur un projet ouvre ce projet dans l'onglet actuellement actif : il remplace donc le contenu de l'onglet courant.
- `Shift + clic` sur un projet ouvre ce projet dans un nouvel onglet.
- `Ctrl + clic` sur un projet ouvre également ce projet dans un nouvel onglet.
- Avec `Shift` ou `Ctrl`, l'onglet actuellement actif ne doit jamais être remplacé ni perdre son contenu.
- Vérifier que le clic modifié ne déclenche pas également le comportement du clic simple.

Examiner la gestion des événements clavier/souris dans iced ainsi que la logique de création, sélection et remplacement des onglets.

### NAV-06-02 — Centrer verticalement les noms des projets dans le panneau gauche

**Priorité : P2**

Dans la liste des projets du panneau de gauche, le nom de chaque projet est actuellement placé en haut à gauche du bouton.

Comportement attendu :
- Conserver l'alignement horizontal à gauche.
- Centrer verticalement le texte dans le bouton.
- Le nom doit donc être placé au milieu à gauche, et non en haut à gauche.

Vérifier le rendu avec différentes hauteurs de boutons et des noms longs.

## 2. Sommaire et panneau de gauche

### SIDEBAR-06-01 — Centrer verticalement les titres du sommaire

**Priorité : P2**

Dans le sommaire du document, les noms des titres sont actuellement alignés en haut à gauche de leurs éléments.

Comportement attendu :
- Conserver l'alignement horizontal à gauche.
- Centrer verticalement le texte dans chaque élément du sommaire.
- Le texte doit être placé au milieu à gauche.

Vérifier les différents niveaux de titres et les éléments dont le texte occupe plusieurs lignes.

### SIDEBAR-06-02 — Placer le sommaire en haut du panneau et supprimer les informations redondantes

**Priorité : P2**

Dans la partie supérieure du panneau de gauche, une catégorie ou un en-tête affiche le nom du document actuel. On y trouve également le nom de fichier `document.tw`. Ces informations sont inutiles dans cette zone.

Comportement attendu :
- Le sommaire doit être le premier élément affiché tout en haut du panneau de gauche.
- Supprimer l'en-tête ou la catégorie qui affiche le nom du document actuel.
- Supprimer l'affichage du nom de fichier `document.tw`.
- Ne pas supprimer les données ou fonctionnalités internes nécessaires à l'ouverture, à l'enregistrement ou à la gestion du document. Il s'agit uniquement de retirer ces éléments de l'interface à cet emplacement.

## 3. Top bar

### TOPBAR-06-01 — Placer le bouton de fermeture à gauche dans la top bar uniquement

**Priorité : P1**

Le bouton permettant de fermer le document ou l'élément actuellement ouvert doit se trouver dans la top bar, à gauche.

Comportement attendu :
- Afficher le bouton de fermeture dans la partie gauche de la top bar.
- Le retirer du panneau de gauche.
- Ne pas afficher de deuxième bouton de fermeture dans le panneau latéral.
- Conserver le comportement fonctionnel de fermeture existant.
- Vérifier que le bouton ferme l'élément attendu sans fermer accidentellement l'application ou un autre onglet.

Ne pas confondre fermeture d'un document/onglet et fermeture de l'application.

### TOPBAR-06-02 — Répartir strictement les icônes en trois groupes équilibrés

**Priorité : P1 — Exigence récurrente, à traiter impérativement**

La top bar présente de nouveau toutes les icônes alignées à gauche. Ce comportement n'est pas souhaité et a déjà fait l'objet de demandes répétées dans les rapports précédents.

La top bar doit être organisée en trois zones distinctes :

1. **À gauche :** les commandes générales et les commandes qui ne relèvent ni de l'édition ni du thème.
2. **Au centre :** toutes les icônes et commandes d'édition du document (mise en forme, listes, titres, etc.).
3. **À droite :** toutes les icônes et commandes liées au thème et à son apparence.

Exigences impératives :
- Les trois groupes doivent être réellement répartis dans la largeur de la top bar.
- Le groupe d'édition doit être centré dans la top bar, et non simplement placé après le groupe de gauche.
- Le groupe de thème doit être aligné à droite.
- Le groupe général doit rester à gauche.
- Ne pas utiliser des espaces fixes ou des marges arbitraires qui casseraient selon la largeur de fenêtre.
- Utiliser une structure de layout robuste et responsive.
- Vérifier différentes tailles de fenêtre et les cas où certains contrôles sont masqués ou indisponibles.

**Critère d'acceptation visuel :** trois groupes clairement distincts : commandes générales à gauche, commandes d'édition au centre, commandes de thème à droite. Aucune de ces familles ne doit se retrouver entièrement regroupée à gauche.

Ne pas déclarer cette tâche terminée sans vérification visuelle dans l'application. Cette exigence est répétée parce que le problème revient malgré les demandes précédentes : il faut donc vérifier la disposition réelle, pas seulement modifier le code en supposant que le résultat sera correct.

## 4. Éditeur de thème

### THEME-06-01 — Supprimer les pastilles de couleur redondantes à droite du bouton du color picker

**Priorité : P2**

Dans l'éditeur de thème, des pastilles de couleur restent affichées à droite du bouton qui ouvre le color picker. Elles auraient dû être retirées, mais sont toujours présentes.

Comportement attendu :
- Examiner leur fonction avant de les retirer.
- Si elles sont purement décoratives ou redondantes, supprimer les pastilles situées à droite du bouton du color picker.
- Ne pas supprimer les contrôles de couleur utiles ni modifier les valeurs de thème enregistrées.

### THEME-06-02 — Ouvrir le color picker en cliquant directement sur la pastille de couleur actuelle

**Priorité : P1**

Le color picker est actuellement ouvert à l'aide d'un bouton séparé, représenté par une icône contenant trois petits points. Ce fonctionnement n'est pas souhaité.

Comportement attendu :
- La pastille qui affiche la couleur actuelle du champ doit elle-même être cliquable.
- Un clic sur cette pastille ouvre le color picker associé à ce champ.
- Le bouton séparé avec les trois petits points ne doit plus être nécessaire et doit être retiré si son unique fonction est d'ouvrir ce color picker.
- Le color picker doit permettre de modifier directement la couleur représentée par la pastille.
- La pastille doit se mettre à jour en temps réel pendant la sélection, sans attendre la fermeture du color picker.
- La valeur du champ et la couleur affichée doivent rester synchronisées.
- La validation, l'annulation et la persistance doivent respecter le modèle déjà utilisé par l'éditeur de thème. Éviter une sauvegarde prématurée ou une perte des changements en cours.
- Vérifier que chaque pastille ouvre le sélecteur de son propre champ et qu'une modification ne change pas accidentellement la couleur d'un autre champ.

## 5. Ordre de traitement suggéré

1. `NAV-06-01` — Gestion des clics simples et modifiés pour les onglets.
2. `TOPBAR-06-02` — Répartition structurelle des trois groupes de commandes.
3. `TOPBAR-06-01` — Déplacement du bouton de fermeture.
4. `THEME-06-02` — Ouverture du color picker depuis la pastille et mise à jour en direct.
5. `THEME-06-01` — Suppression des pastilles redondantes.
6. `SIDEBAR-06-02` — Simplification du haut du panneau et mise en avant du sommaire.
7. `NAV-06-02` et `SIDEBAR-06-01` — Centrage vertical des textes.

Cet ordre est indicatif. Si l'analyse du code révèle des dépendances différentes, l'adapter et documenter brièvement la raison.

## 6. Vérifications et définition de terminé

Le rapport est traité lorsque :
- Un clic simple sur un projet remplace le contenu de l'onglet actif.
- `Shift + clic` et `Ctrl + clic` ouvrent chacun un nouvel onglet sans remplacer l'onglet actif.
- Les noms de projets et les titres du sommaire sont alignés à gauche et centrés verticalement.
- Le sommaire apparaît en haut du panneau gauche, sans l'en-tête du nom de document ni `document.tw`.
- Le bouton de fermeture est présent uniquement dans la partie gauche de la top bar, et sa portée est correcte.
- La top bar affiche distinctement les commandes générales à gauche, les commandes d'édition au centre et les commandes de thème à droite.
- Les pastilles redondantes à droite du bouton de color picker ont été retirées après vérification de leur fonction.
- Le clic sur la pastille de couleur ouvre le color picker correspondant.
- La pastille et la valeur du champ se mettent à jour en direct pendant la sélection.
- Les tests pertinents sont passés et les comportements voisins n'ont pas régressé.

Mettre à jour `IMPLEMENTATION_PROGRESS.md` avec les identifiants de ce rapport, les décisions prises, les fichiers modifiés et les vérifications effectuées.
