# Feuille de route révisée — Typst Writer

L'objectif est de sécuriser le socle avant de développer l'éditeur WYSIWYG, sans remettre en cause la syntaxe `.tw` ni le moteur Typst.

## Jalon 0 · Fiabiliser le langage et le pipeline

- Formaliser la grammaire : règles de priorité, imbrication des éléments inline, délimiteurs incomplets, listes multilignes et marqueurs mal formés.
- Garantir le round-trip : `parse(serialiser(AST)) == AST` pour tout AST valide.
- Tester l'échappement Typst par contexte : texte, code, liens, couleurs, notes et blocs bruts.
- Définir la stabilité textuelle : une ouverture et une sauvegarde sans modification ne doivent pas reformater inutilement le fichier.
- Ajouter des tests de compilation couvrant chaque construction de la syntaxe gelée.
- Sécuriser l'assemblage : préserver les inclusions nécessaires, éviter les bibliographies dupliquées et conserver les styles explicites.

## Jalon 1 · Stabiliser le modèle d'édition

- Définir la propriété des données : `document.tw` est la source persistante ; l'AST est la représentation sémantique utilisée par les fonctionnalités de l'éditeur.
- Fiabiliser la synchronisation texte/AST, y compris pendant les saisies incomplètes.
- Gérer les diagnostics : erreurs de syntaxe, constructions non reconnues et erreurs de génération, sans perte de contenu.
- Implémenter les tests d'édition : curseur, sélection, collage, formatage, annulation et rétablissement.
- Rendre les actions de formatage atomiques : une action correspond à une opération cohérente dans l'historique.
- Conserver la complétion bibliographique locale et garantir que les modifications de texte ne corrompent pas les clés `@clé`.

## Jalon 2 · Migration, plan et document unique

- Créer un rapport de migration : chapitres détectés, ordre, éléments convertis, blocs bruts et avertissements.
- Préserver les dépendances : images, fichiers inclus, macros, références croisées et chemins relatifs.
- Tester la fidélité du rendu avant/après migration sur des projets représentatifs.
- Conserver intégralement les sources originales et rendre la migration non destructive et idempotente.
- Attribuer des identifiants stables aux blocs pour préserver la navigation malgré les insertions, suppressions et déplacements.
- Construire le plan navigable avec accès aux titres et retour à leur position source.
- Définir les limites des blocs Typst bruts : contenu opaque par défaut, avec indication claire dans l'interface.
- Unifier le compteur de mots sur le document actif, en documentant le traitement des blocs bruts.

## Jalon 3 · Rendu, pagination et performances

- Séparer composition et rasterisation : le zoom ne doit pas déclencher inutilement une recompilation Typst.
- Versionner les rendus et ignorer les résultats devenus obsolètes.
- Borner le cache d'images et limiter les tâches de rendu simultanées.
- Gérer proprement l'invalidation des images lorsque l'échelle ou le contenu change.
- Tester les modes d'affichage : ajustement en largeur, en hauteur et zoom libre, y compris sur les documents longs.

## Jalon 4 · Éditeur de blocs WYSIWYG

- Remplacer progressivement le widget texte par une interface de blocs reliée à l'AST.
- Préserver curseur, sélection et historique lors des transformations de blocs.
- Synchroniser les modifications entre l'éditeur, l'AST, le plan et l'aperçu.
- Maintenir un mode d'accès au Typst brut pour les constructions avancées.
- Valider les performances sur des documents longs avant de généraliser le nouvel éditeur.

## Ordre de priorité

P0

Avant toute extension majeure

Grammaire, round-trip, échappement, tests de compilation et migration non destructive.

P1

Avant le WYSIWYG

Synchronisation texte/AST, curseur, undo/redo, identifiants stables et diagnostics.

&#x20;P2&#x20;

Avant la finalisation

Cache de rendu, zoom, navigation complète, performances et finitions du WYSIWYG.

Principe directeur : ne pas ajouter de fonctionnalités au langage pour l'instant. Consolider les garanties du langage existant, fiabiliser l'édition et la migration, puis développer l'interface structurée sur cette base.