# Bug report – Améliorations et fonctionnalités

## 1. Normes et structure documentaire

- Ajouter une option pour insérer un saut de page avant la bibliographie.
- Permettre de définir la bibliographie comme un titre de niveau 1.
- Ajouter une balise `[[TOC]]` (ou équivalent) permettant de générer automatiquement une table des matières via un bouton.

## 2. Left panel

- Sommaire : corriger l'espacement des lignes hiérarchiques. Les lignes doivent rester visuellement continues, sans espace au niveau des bordures du panneau. Conserver l'indentation des éléments en allongeant les lignes.
- Bibliographie : permettre de replier les entrées individuelles. La recherche bibliographique doit continuer à fonctionner et afficher les résultats même lorsque la section est repliée.
- Projets : conserver la section, mais y intégrer l'accès à la bibliothèque (vue étagères) et permettre de créer, renommer et supprimer des projets directement depuis cette interface.

## 3. Listes à plusieurs niveaux

- Implémenter les listes à puces et numérotées multiniveaux.
- `Tab` : augmenter le niveau d'indentation.
- `Ctrl + Tab` ou `Shift + Tab` : diminuer le niveau d'indentation.
- Ajouter l'autocomplétion des listes numérotées : `1. ` déclenche une liste et `Enter` crée automatiquement l'élément suivant.
- Ajouter la gestion des niveaux pour les listes numérotées.
- `Ctrl + Shift + ↑ / ↓` : déplacer un élément de liste vers le haut ou vers le bas.

## 4. Métadonnées et profils

- Permettre de définir plusieurs profils de métadonnées.
- Autoriser des valeurs différentes pour une même métadonnée selon le profil (ex. auteur : Jean / Jacques).
- Les balises associées doivent utiliser les valeurs du profil actif.
- Permettre le remplacement automatique des balises de métadonnées dans le texte par leurs valeurs respectives.
- Réserver les noms de balises système (glossaire, couleur, etc.) afin d'éviter les conflits avec les métadonnées personnalisées.

## 5. Top bar et ergonomie

- Remplacer les boutons textuels `BIB` et `GLO` par des icônes.
- Remplacer la liste déroulante des styles de titres par une icône ouvrant un menu de sélection.
- Remplacer la liste déroulante du glossaire par une icône ouvrant une fenêtre dédiée permettant de consulter, ajouter, modifier et supprimer des entrées.
- Revoir la gestion des couleurs :
  - Distinguer clairement le choix d'une couleur personnalisée et son application.
  - Conserver un accès rapide à la dernière couleur enregistrée pour insérer directement sa balise.
- Ajouter une fine ligne de séparation entre les fenêtres lorsque plusieurs documents ou vues sont ouverts simultanément.

## 6. Rendu et compilation

- Corriger les erreurs de compilation liées aux bibliographies dupliquées.
- Améliorer la synchronisation entre le mode édition et le mode rendu : actualisation plus fluide et systématiquement cohérente du rendu lors des modifications.