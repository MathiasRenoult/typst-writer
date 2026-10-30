# Cahier d'observations et d'améliorations — application d'édition

> **Objectif :** transformer les observations utilisateur en exigences compréhensibles, vérifiables et directement exploitables par un agent de développement.
>
> **Consigne générale :** ce document décrit le comportement attendu, pas une architecture à imposer. Avant de modifier le code, inspecter l'implémentation existante, reproduire les problèmes et identifier les composants, états et fichiers concernés. Préserver les fonctions qui ne sont pas visées. Ne pas corriger un problème fonctionnel par un simple contournement visuel.

## 0. Méthode de travail et priorités

### Méthode recommandée

1. **Diagnostiquer avant de modifier.** Reproduire le problème et identifier sa cause probable dans le code.
2. **Évaluer les répercussions.** Vérifier les effets sur les autres documents, projets, onglets, fenêtres, thèmes et normes.
3. **Procéder par lots cohérents.** Éviter une refonte massive mêlant bugs, ergonomie et nouvelles fonctionnalités.
4. **Traiter explicitement les ambiguïtés.** Lorsqu'une demande autorise plusieurs comportements, ne pas en choisir un arbitrairement : relever le point et proposer une décision.
5. **Préserver les données.** Une opération échouée ne doit jamais remplacer silencieusement une donnée valide par une donnée invalide ou par une valeur par défaut.
6. **Tester les cas limites.** Notamment : listes longues, plusieurs fenêtres, renommages, fichiers invalides, changements de thème, valeurs partiellement saisies et documents volumineux.
7. **Ajouter ou adapter les tests.** Toute modification de logique ou de format de données doit être accompagnée de tests pertinents et, si nécessaire, d'une mise à jour de la documentation.

### Échelle de priorité

- **P0 — Intégrité des données :** risque de perte, corruption, remplacement ou verrouillage injustifié.
- **P1 — Fonctionnement :** comportement incorrect, incohérent ou bloquant.
- **P2 — Ergonomie :** organisation, lisibilité et interactions.
- **P3 — Évolution :** nouvelle fonctionnalité ou perfectionnement non bloquant.

Les priorités ci-dessous sont indicatives. L'agent peut les ajuster après diagnostic, mais doit expliquer tout changement important.

---

# 1. Thèmes

## THEME-01 — Empêcher la perte ou le remplacement d'un thème lors de son enregistrement
**Priorité : P0 — Intégrité des données**

**Problème observé**  
Lors de la modification de certaines valeurs d'un thème, cliquer sur « Enregistrer » semble supprimer ou rendre inutilisable le thème modifié. L'application charge alors un autre thème, apparemment le premier de la liste (« sombre »).

Erreurs notamment observées :
- `everforest-sombre.toml` : `colors.background` et `colors.surface` sont identiques ; le message indique que la marge de la page ne serait plus visible.
- `gruvbox-nuit.toml` : même message.

**Comportement attendu**
- L'enregistrement ne doit jamais supprimer, remplacer ou altérer silencieusement un autre thème.
- En cas de validation échouée, conserver le fichier d'origine et le thème actuellement appliqué.
- Expliquer précisément la propriété invalide et permettre à l'utilisateur de la corriger.
- Ne pas charger automatiquement un thème sans rapport à la suite d'une erreur.
- Distinguer les erreurs bloquantes des simples avertissements de contraste ou de lisibilité.

**Vérifications** : tester plusieurs thèmes existants, des valeurs invalides et deux couleurs identiques. Confirmer qu'un échec d'enregistrement ne modifie ni le fichier source ni le thème actif.

---

# 2. Panneau latéral gauche

## LEFT-01 — Déplacer le contrôle de réduction du panneau
**Priorité : P2**

Le bouton permettant de réduire ou masquer le panneau latéral est actuellement placé dans la barre supérieure, à un endroit peu cohérent. Le déplacer dans le panneau lui-même, à un emplacement naturellement associé à cette fonction.

## LEFT-02 — Supprimer le bloc d'informations redondant
**Priorité : P2**

Supprimer le premier bloc situé en haut du panneau. Il affiche le nom du document, déjà indiqué dans l'onglet, ainsi que le nombre de documents, qui vaut actuellement toujours `1`. Ne conserver aucune information redondante dans l'interface actuelle.

## LEFT-03 — Supprimer la section des anciens chapitres archivés
**Priorité : P2**

Supprimer la section « anciens chapitres conservés en archive », jugée inutile dans le flux de travail actuel. Vérifier au préalable qu'elle ne porte pas une fonction de récupération ou de conservation nécessaire ailleurs.

## LEFT-04 — Représenter le sommaire sous forme d'arborescence
**Priorité : P2**

Remplacer les séparateurs horizontaux par une représentation hiérarchique avec des lignes de connexion verticales et horizontales, proche d'une arborescence de terminal ou de fils de commentaires.

Comportement attendu :
- rendre les relations parent-enfant et les niveaux immédiatement lisibles ;
- permettre de réduire et développer une branche ;
- masquer tous les descendants d'une branche réduite sans modifier le document ;
- conserver une indication claire du titre sélectionné ;
- éviter toute ambiguïté entre le contrôle de réduction et l'action de naviguer vers un titre.

## LEFT-05 — Afficher toutes les collections dans le sélecteur
**Priorité : P1**

**Problème observé :** le panneau permettant de choisir une collection semble n'en afficher qu'une partie. La recherche retrouve les collections manquantes et leur ordre semble alphabétique ; le problème pourrait donc venir de l'affichage ou du défilement.

**Comportement attendu :**
- toutes les collections disponibles doivent pouvoir être parcourues sans recherche ;
- le défilement vertical doit fonctionner lorsque la liste dépasse la hauteur du panneau ;
- la recherche doit filtrer l'ensemble des collections, pas uniquement celles déjà rendues à l'écran ;
- conserver le tri alphabétique actuel, sauf contrainte technique justifiée.

## LEFT-06 — Clarifier l'intitulé de la bibliographie
**Priorité : P2**

Remplacer l'affichage du nom technique `references.bib` par un intitulé destiné à l'utilisateur, par exemple **« Entrées bibliographiques »**. Le fichier peut conserver son nom technique en interne.

## LEFT-07 — Corriger le comptage des entrées bibliographiques
**Priorité : P1**

**Problème observé :** le nombre affiché semble plafonner à `99`, alors qu'un import depuis Zotero contenait environ 137 entrées.

**Comportement attendu :**
- afficher le nombre réel d'entrées ;
- vérifier si le plafonnement vient de l'affichage, du type numérique, du traitement du fichier ou de l'import ;
- confirmer que toutes les entrées sont réellement présentes et exploitables, et pas seulement que le compteur est corrigé.

## LEFT-08 — Afficher l'année de publication dans la bibliothèque
**Priorité : P2**

Ajouter l'année de publication à chaque entrée bibliographique. La présentation doit être compacte et lisible : l'année doit être facile à repérer, mais visuellement secondaire par rapport au titre et à l'auteur. Prévoir un affichage cohérent lorsque l'année est absente ou inconnue.

## LEFT-09 — Ajouter des filtres à la bibliothèque
**Priorité : P2**

Permettre de filtrer les entrées bibliographiques selon :
- le nom de l'auteur ;
- le titre ;
- l'année de publication.

Les critères doivent pouvoir être utilisés séparément ou combinés. Indiquer lorsqu'un filtre est actif et permettre de réinitialiser facilement les critères. La recherche doit porter sur toutes les entrées.

## LEFT-10 — Clarifier ou supprimer l'information de l'onglet « Document »
**Priorité : P3**

L'onglet « Document » affiche `document.tw`, suivi d'un nombre dont la signification n'est pas claire.

- Identifier ce que représente ce nombre.
- S'il est utile, l'afficher avec un libellé explicite et une valeur compréhensible.
- S'il n'apporte aucune information utile, supprimer cette section.
- Ne pas exposer de métadonnée technique sans explication.

## LEFT-11 — Afficher les actions d'un projet uniquement au survol
**Priorité : P2**

Masquer par défaut les boutons de renommage et de suppression. Les afficher au survol de la ligne du projet. Ils doivent également rester accessibles au clavier et leur apparition ne doit pas déplacer les autres éléments de la ligne.

## LEFT-12 — Fiabiliser le renommage des projets
**Priorité : P0 — Intégrité et gestion des projets**

**Problèmes observés :**
1. Après le renommage, le nom du projet dans l'onglet situé en bas à gauche n'est pas actualisé immédiatement. Un enregistrement manuel est nécessaire.
2. Après le renommage, l'application affiche parfois : « Une autre instance travaille sur ce projet (processus 86779). Projet en lecture seule. »

**Comportement attendu :**
- propager immédiatement le nouveau nom à tous les composants qui l'affichent ;
- ne pas exiger un enregistrement manuel supplémentaire ;
- ne pas créer de faux conflit de verrouillage ;
- distinguer une véritable autre instance d'un verrou obsolète ou d'un état interne non actualisé ;
- en cas de conflit réel, expliquer la situation sans compromettre les données.

**Vérifications :** tester le renommage avec un projet ouvert dans une seule fenêtre, puis dans plusieurs fenêtres ou instances si ce scénario est pris en charge. Vérifier également la fermeture et la réouverture du projet.

---

# 3. Éditeur

## EDITOR-01 — Unifier la couleur de fond de l'éditeur et celle de l'application
**Priorité : P1**

**Problème observé :** la couleur de fond de l'éditeur diffère parfois légèrement de celle du reste de l'application.

**Comportement attendu :** identifier la source de la différence (palette, transparence, rendu, style local, etc.) et utiliser la couleur prévue par le thème pour les surfaces concernées. Vérifier le résultat après changement de thème et dans les fenêtres actives et inactives. Ne pas introduire une couleur codée en dur qui contournerait le système de thèmes.

## EDITOR-02 — Supprimer un mot avec Ctrl+Suppr et Ctrl+Del
**Priorité : P1**

Les raccourcis `Ctrl+Suppr` et `Ctrl+Del` doivent supprimer le mot situé après le curseur, et non un seul caractère. Vérifier la cohérence avec les conventions du système et le clavier utilisé.

## EDITOR-03 — Transformer `---` en séparateur horizontal
**Priorité : P2**

Une ligne contenant uniquement `---` doit produire un séparateur horizontal dans le document rendu.

**Points à définir et tester :**
- déclenchement après validation de la ligne (par exemple avec Entrée) ou au moment de la compilation ;
- comportement si `---` apparaît au milieu d'un paragraphe ;
- possibilité d'afficher littéralement trois tirets ;
- cohérence avec la personnalisation du séparateur dans les normes (NORMS-10).

## EDITOR-04 — Interpréter trois lignes vides comme un saut de page
**Priorité : P2**

Trois lignes vides consécutives doivent produire un saut de page dans le rendu.

**À préciser avant implémentation :** déterminer si cela signifie trois lignes vides ou trois retours à la ligne ; définir si cette convention relève de la syntaxe source ou du rendu ; prévoir un moyen non ambigu d'afficher et de modifier le saut ; éviter qu'une succession ordinaire de lignes vides entraîne accidentellement un saut difficile à supprimer.

## EDITOR-05 — Ajouter l'autocomplétion et l'autoformatage des tableaux
**Priorité : P2**

Améliorer la saisie des tableaux avec :
- une autocomplétion adaptée à la syntaxe de tableau déjà prise en charge ;
- un autoformatage qui produit une structure régulière et lisible ;
- une gestion cohérente des cellules, colonnes et lignes lors de l'ajout ou de la suppression de contenu.

Respecter la syntaxe existante. Ne pas introduire une syntaxe concurrente sans nécessité.

## EDITOR-06 — Ajouter les mises en forme souligné et barré
**Priorité : P2**

Ajouter une syntaxe pour le souligné (`underline`) et le barré (`strikethrough`). Définir les délimiteurs en cohérence avec les syntaxes déjà présentes. Le formatage doit fonctionner sur une sélection et ne pas produire de balises mal équilibrées.

## EDITOR-07 — Ajouter une syntaxe de commentaires ignorés par les compilateurs
**Priorité : P3**

Ajouter une syntaxe de commentaire conservée dans le fichier de travail, mais ignorée dans le rendu et par les compilateurs concernés.

Définir avant implémentation :
- les délimiteurs ;
- la prise en charge des commentaires sur une ligne et sur plusieurs lignes ;
- le comportement des commentaires imbriqués ;
- leur éventuelle coloration dans l'éditeur.

Éviter les conflits avec la syntaxe existante et celle de Typst.

## EDITOR-08 — Formater les siècles et millénaires en petites capitales
**Priorité : P2**

Permettre le formatage automatique en petites capitales des nombres romains désignant des siècles ou des millénaires (ex. `XXe`, `IIe`, `Ier`).

- Le comportement doit être défini dans les normes typographiques.
- Il doit être activé par défaut en français et en anglais.
- L'utilisateur doit pouvoir le désactiver.
- Ne pas transformer indistinctement tous les nombres romains : la règle concerne les désignations de siècles et de millénaires.
- Tester notamment `Ier`, `IIe`, `XXe` et les variantes de casse saisies.

## EDITOR-09 — Rendre la barre de défilement plus discrète
**Priorité : P2**

La barre de défilement actuelle est trop visible. La rendre beaucoup plus discrète, voire la masquer visuellement, à condition que le défilement reste évident et accessible. Ne pas supprimer la possibilité de faire défiler le contenu ni les indications de position réellement utiles.

## EDITOR-10 — Ajouter des raccourcis pour les aides visuelles
**Priorité : P2**

Ajouter des raccourcis permettant d'activer ou de désactiver :
- la numérotation des lignes ;
- l'affichage des caractères invisibles : espaces, tabulations, retours à la ligne, etc.

L'état de ces options doit être visible. Leur activation ne doit jamais modifier le contenu du document.

---

# 4. Onglets et fenêtres

## WINDOWS-01 — Rendre le changement de focus entre fenêtres prévisible
**Priorité : P1**

Le focus doit pouvoir passer d'une fenêtre à une autre sans devoir systématiquement cliquer dans la fenêtre cible. Étudier le fonctionnement actuel et définir un comportement cohérent avec le système de fenêtres de l'application et les conventions du système d'exploitation.

Ne pas déclencher un changement de focus involontaire lorsque l'utilisateur survole simplement une fenêtre.

## WINDOWS-02 — Respecter les règles de clic pour l'ouverture des documents
**Priorité : P1**

Les règles souhaitées sont :
- **Clic simple :** ouvrir le document dans l'onglet actuel ; si le document est déjà ouvert, basculer vers son onglet existant.
- **Maj + clic :** ouvrir le document dans un nouvel onglet.

**Ambiguïté à résoudre :** la note initiale indique également que Maj + clic devrait basculer vers l'onglet existant si le document est déjà ouvert. Il faut donc décider si Maj + clic doit toujours créer un nouvel onglet ou s'il doit réutiliser l'onglet existant dans ce cas. Ne pas implémenter une interprétation arbitraire.

Le comportement actuel est décrit comme aléatoire : tester chaque cas et rendre les règles strictes et reproductibles.

## WINDOWS-03 — Prévisualiser la destination pendant le glisser-déposer d'un onglet
**Priorité : P2**

Lorsqu'un onglet est déplacé, afficher un aperçu visuel (« fantôme ») de la destination probable avant le relâchement du clic.

- rendre visible la fenêtre ou la zone de destination ;
- distinguer le déplacement vers une fenêtre existante du détachement créant une nouvelle fenêtre ;
- actualiser l'aperçu pendant le déplacement du pointeur ;
- ne réaliser l'opération qu'au relâchement du clic.

## WINDOWS-04 — Faciliter le détachement d'un onglet pour créer une fenêtre
**Priorité : P2**

La création d'une nouvelle fenêtre ne doit pas exiger que le pointeur atteigne exactement le bord de l'écran. Une zone de déclenchement plus accessible doit distinguer clairement le détachement d'un onglet du déplacement vers une autre fenêtre.

L'intention exprimée est qu'un déplacement ayant franchi environ les deux tiers de la fenêtre puisse suffire à déterminer la création d'une nouvelle fenêtre. Définir précisément cette zone et la coordonner avec l'aperçu de WINDOWS-03 pour éviter les déclenchements accidentels.

---

# 5. Barre supérieure

## TOPBAR-01 — Supprimer le nom technique du document
**Priorité : P2**

Supprimer la mention `document.tw` située à gauche de la barre supérieure. Le nom du document étant déjà affiché dans les onglets, cette mention est redondante.

## TOPBAR-02 — Réorganiser les groupes de boutons
**Priorité : P2**

Réorganiser les icônes en groupes fonctionnels clairement identifiables et alignés à gauche, au centre ou à droite selon leur rôle. Éviter que toutes les commandes soient empilées dans un seul groupe à gauche. La priorité est une hiérarchie fonctionnelle lisible, pas un alignement décoratif.

## TOPBAR-03 — Remplacer les commandes de gestion de fichiers par un panneau d'import
**Priorité : P3 — Nouvelle fonctionnalité**

La gestion actuelle des projets étant locale à l'application, les boutons de gestion de fichiers présents dans la barre supérieure ne sont plus utiles dans leur forme actuelle.

**Comportement souhaité :**
- remplacer ces boutons par une commande unique ouvrant un panneau dédié à l'import de fichiers externes ;
- prendre en charge les fichiers `.tw` dans un premier temps ;
- prévoir à terme l'import de fichiers Typst ;
- expliquer les formats pris en charge et afficher des erreurs d'import compréhensibles.

Ne pas confondre l'import d'un fichier externe avec l'ouverture ou la gestion d'un projet local. Définir le comportement en cas de nom identique à un document existant avant d'implémenter.

## TOPBAR-04 — Corriger l'alignement et le choix des icônes de mise en forme
**Priorité : P2**

Centrer chaque icône dans son bouton. Remplacer celles qui représentent mal leur action. Conserver une cohérence de taille, d'épaisseur, de style et d'alignement. Vérifier le résultat à différentes tailles de fenêtre.

## TOPBAR-05 — Rendre le menu des niveaux de titre utilisable
**Priorité : P1**

Le panneau de sélection des niveaux de titre se ferme dès que le pointeur quitte le bouton, avant que l'utilisateur puisse atteindre le panneau.

Corriger la gestion du survol et du focus afin que le panneau reste ouvert pendant le déplacement du pointeur du bouton vers le menu. Permettre également son utilisation au clavier.

## TOPBAR-06 — Repenser la commande de couleur
**Priorité : P2**

Remplacer le fonctionnement actuel par deux commandes :

1. **Couleur courante :** afficher une pastille représentant la dernière couleur choisie. Un clic applique cette couleur au texte sélectionné en insérant la syntaxe correspondante.
2. **Sélection de couleur :** un bouton voisin ouvre un sélecteur de couleur.

Le sélecteur doit proposer un color picker, une palette de couleurs de base et une indication de la couleur choisie. La nouvelle couleur devient ensuite la couleur courante du premier bouton. Définir un comportement cohérent lorsqu'aucun texte n'est sélectionné.

## TOPBAR-07 — Supprimer les boutons d'aperçu obsolètes
**Priorité : P2**

Supprimer les boutons d'aperçu devenus obsolètes. Vérifier qu'aucune fonction encore utilisée n'est accessible uniquement par ces boutons avant de les retirer.

## TOPBAR-08 — Déplacer et revoir les commandes de zoom
**Priorité : P2**

Les quatre boutons actuels de réglage du zoom sont jugés peu esthétiques et mal placés.

- les remplacer par un contrôle compact et cohérent ;
- centrer correctement les icônes ;
- placer les commandes directement dans la fenêtre de rendu et non dans la barre supérieure ;
- conserver les fonctions de zoom avant, zoom arrière et réinitialisation, ainsi que toute autre fonction actuellement disponible.

## TOPBAR-09 — Aligner les commandes de thème à droite
**Priorité : P2**

Placer les commandes relatives aux thèmes dans la partie droite de la barre supérieure, au sein d'un groupe identifiable.

## TOPBAR-10 — Supprimer les commandes manuelles de gestion du dossier des thèmes
**Priorité : P2**

- Supprimer « Recharger le dossier des thèmes » : la liste doit s'actualiser automatiquement à l'ouverture du sélecteur ou lorsqu'un changement pertinent est détecté.
- Supprimer « Ouvrir le dossier des thèmes » : l'application dispose déjà d'un éditeur de thèmes.

L'actualisation automatique ne doit pas recharger intempestivement le thème actuellement appliqué.

## TOPBAR-11 — Faire basculer les thèmes favoris dans toutes les fenêtres
**Priorité : P1**

Remplacer le bouton actuel clair/sombre/système par un bouton qui bascule entre deux thèmes favoris définis par l'utilisateur : un thème sombre et un thème clair.

Le changement doit s'appliquer à toutes les fenêtres de l'application, y compris celles qui ne sont pas actives, et pas seulement aux thèmes par défaut.

**Points à définir :**
- où l'utilisateur choisit ses deux thèmes favoris ;
- le comportement si aucun favori n'est défini ou si un thème favori a été supprimé ;
- le maintien ou la suppression du mode « système ».

## TOPBAR-12 — Replacer le bouton de la bibliothèque
**Priorité : P2**

Déplacer le bouton de la bibliothèque vers un emplacement cohérent avec sa fonction et l'organisation générale de la barre supérieure. Éviter de le placer arbitrairement parmi les commandes de mise en forme.

## TOPBAR-13 — Regrouper l'éditeur de thèmes avec les commandes de thème
**Priorité : P2**

Placer le bouton ouvrant l'éditeur de thèmes à proximité des autres commandes relatives aux thèmes, dans le même groupe fonctionnel.

---

# 6. Éditeur de thèmes

## THEME-EDITOR-01 — Corriger l'édition des valeurs hexadécimales
**Priorité : P1**

**Problème observé :** lors de l'édition d'une valeur hexadécimale, la touche de suppression déplace le curseur, mais les caractères ne sont pas supprimés.

**Comportement attendu :** le champ doit permettre insertion, suppression, sélection partielle, remplacement et collage. La validation ne doit pas empêcher l'édition intermédiaire d'une valeur temporairement incomplète.

## THEME-EDITOR-02 — Ajouter des sélecteurs de couleur
**Priorité : P2**

Ajouter un color picker aux champs de couleur. Conserver également la possibilité de saisir directement une valeur hexadécimale.

## THEME-EDITOR-03 — Clarifier la fonction des pastilles de couleur
**Priorité : P2**

Les pastilles situées à gauche des champs ne sont pas compréhensibles. Expliquer leur fonction par un libellé, une infobulle ou une interaction évidente. Les supprimer si elles n'ont aucune utilité fonctionnelle.

## THEME-EDITOR-04 — Garantir qu'une modification ne corrompt jamais un thème
**Priorité : P0 — Intégrité des données**

**Problème observé :** la modification d'un thème peut parfois le rendre inutilisable.

**Comportement attendu :**
- valider les changements avant de remplacer le fichier existant ;
- préserver le dernier état valide en cas d'erreur ;
- ne jamais écraser un thème valide par un fichier invalide ;
- présenter une erreur explicite et permettre de revenir à la dernière version valide ;
- éviter les écritures partielles, en privilégiant une stratégie d'enregistrement atomique adaptée au système de fichiers ;
- vérifier les champs obligatoires et les types de valeurs avant l'enregistrement.

Cette exigence est directement liée à THEME-01.

## THEME-EDITOR-05 — Revoir et polir l'interface de l'éditeur de thèmes
**Priorité : P2**

Effectuer une révision générale de cette fenêtre : hiérarchie visuelle, regroupement des propriétés, homogénéité des champs et contrôles, libellés compréhensibles, color pickers, messages d'erreur et aperçu du résultat.

Le polissage ne doit pas masquer les propriétés avancées ni rendre plus difficile l'accès aux valeurs précises.

---

# 7. Mode focus

## FOCUS-01 — Masquer la barre inférieure
**Priorité : P2**

La barre inférieure (« down-bar ») ne doit pas être visible en mode focus. Vérifier que les fonctions indispensables restent accessibles par raccourci ou après une sortie temporaire du mode focus.

## FOCUS-02 — Centrer le rendu
**Priorité : P2**

Centrer le rendu dans l'espace disponible en mode focus. Le centrage doit rester correct lorsque la fenêtre est redimensionnée ou que le zoom change.

---

# 8. Fenêtre des normes typographiques

## NORMS-01 — Corriger le chevauchement de la barre de défilement
**Priorité : P1**

La barre de défilement empiète sur les boutons de la fenêtre. Corriger la disposition afin que le contenu défilant et les commandes disposent de zones distinctes. Aucun bouton ne doit être masqué ou partiellement recouvert.

## NORMS-02 — Empêcher les interactions de prévisualisation au mauvais endroit
**Priorité : P1**

Lorsque le pointeur survole la fenêtre des normes à proximité d'une séparation entre deux fenêtres, la prévisualisation apparaît et le curseur change alors qu'aucun élément interactif n'est visé.

La prévisualisation et le changement de curseur ne doivent se déclencher que lorsque le pointeur se trouve réellement sur l'élément concerné. Vérifier la propagation des événements et les zones de détection.

## NORMS-03 — Adapter le curseur aux éléments interactifs
**Priorité : P2**

Le curseur doit indiquer clairement lorsqu'il se trouve sur un élément interactif (bouton, lien, contrôle, etc.). Utiliser les conventions habituelles et ne pas afficher le curseur interactif sur de simples zones de contenu.

## NORMS-04 — Repenser la configuration des niveaux de titres
**Priorité : P2**

Pour chaque niveau de titre, permettre de définir séparément :
- la casse ;
- la mise en forme (gras, italique, souligné, etc., selon les fonctions prises en charge) ;
- la taille ;
- la couleur ;
- l'espacement.

Présenter ces propriétés dans une interface claire et cohérente, avec un aperçu du résultat pour chaque niveau. Les options doivent être effectivement appliquées au rendu et sauvegardées dans les normes.

## NORMS-05 — Placer les commandes de chargement et d'enregistrement en haut
**Priorité : P2**

Déplacer les boutons de chargement et d'enregistrement d'une norme en haut de la fenêtre, dans une zone d'actions clairement identifiable.

## NORMS-06 — Indiquer la norme active et son état de modification
**Priorité : P1**

Afficher clairement :
- la norme actuellement utilisée ;
- si elle a été modifiée depuis son chargement ;
- si les modifications sont enregistrées ou non.

Lors de l'enregistrement, demander explicitement si l'utilisateur souhaite écraser la norme originale ou enregistrer les modifications comme une nouvelle variante ou une copie. Ne jamais écraser silencieusement une norme originale. Identifier clairement le fichier ou la norme concernée dans le dialogue.

## NORMS-07 — Configurer les types de guillemets par niveau
**Priorité : P2**

Ajouter un menu permettant de choisir les guillemets utilisés à chaque niveau d'imbrication d'une citation.

- proposer plusieurs conventions typographiques courantes ;
- permettre la saisie de guillemets personnalisés ;
- imposer exactement deux caractères par paire : un ouvrant et un fermant ;
- afficher un aperçu de l'imbrication résultante ;
- éviter toute confusion entre les niveaux successifs.

## NORMS-08 — Définir l'ordre des guillemets français
**Priorité : P2**

Dans les normes françaises, utiliser par défaut l'ordre suivant :
1. `« … »`
2. `‹ … ›`
3. `“ … ”`

Cet ordre doit s'appliquer aux niveaux successifs et rester personnalisable via NORMS-07.

## NORMS-09 — Ajouter les normes allemandes
**Priorité : P2**

Ajouter une norme linguistique allemande et prendre en compte ses particularités typographiques, notamment l'ordre des guillemets imbriqués souhaité :
1. `„ … “`
2. `‚ … ‘`
3. `» … «`

Vérifier également les autres conventions allemandes pertinentes pour les fonctions déjà prises en charge. Ne pas limiter la norme au remplacement des guillemets : elle doit être cohérente avec la langue allemande.

## NORMS-10 — Ajouter les paramètres des séparateurs horizontaux
**Priorité : P2**

Dans la section « Décor », ajouter les propriétés relatives aux lignes horizontales créées par la syntaxe `---` (EDITOR-03).

Permettre de configurer les propriétés visuelles réellement prises en charge par le moteur de rendu et proposer des valeurs par défaut cohérentes avec le thème.

## NORMS-11 — Enrichir les Quick Parts d'en-tête et de pied de page
**Priorité : P3**

Ajouter des champs prédéfinis utilisables dans les en-têtes et pieds de page, notamment :
- nom du document ;
- date ;
- prénom et nom de l'auteur ;
- école ou institution ;
- version du document.

Étudier également l'ajout du numéro de page, du nombre total de pages, du titre du projet ou de la date de dernière modification, si ces informations sont disponibles de manière fiable. Les champs censés être dynamiques doivent se mettre à jour automatiquement et ne pas être insérés comme valeurs figées.

---

# 9. Rechercher et remplacer

## FIND-01 — Ajouter le choix entre recherche normale et expressions régulières
**Priorité : P2**

Permettre de choisir entre une recherche textuelle normale et une recherche par expressions régulières (regex). Le mode actif doit être visible. En mode regex, signaler les expressions invalides sans faire échouer l'éditeur.

## FIND-02 — Mettre en évidence les correspondances
**Priorité : P2**

Mettre en évidence dans l'éditeur les occurrences correspondant à la recherche. Distinguer l'occurrence actuellement sélectionnée des autres correspondances. Actualiser les surlignages lorsque la recherche ou le contenu change, et les retirer lorsque la recherche est effacée ou fermée.

## FIND-03 — Corriger le dépassement des textes indicatifs
**Priorité : P1**

Le texte indicatif (placeholder) dépasse actuellement les dimensions de certains champs. Adapter sa taille, sa troncature ou la disposition des champs pour qu'il reste dans les limites disponibles, y compris lorsque la fenêtre est étroite.

---

# 10. Rendu

## RENDER-01 — Augmenter la limite de zoom
**Priorité : P2**

La limite actuelle de 200 % est trop restrictive. Permettre un agrandissement supérieur.

- définir une limite maximale suffisamment élevée pour la lecture et la vérification ;
- éviter les valeurs qui provoquent des débordements ou rendent l'interface inutilisable ;
- vérifier la fluidité et le positionnement du document ;
- permettre, si possible, la saisie directe d'un pourcentage.

La valeur maximale exacte doit être déterminée après vérification des contraintes du moteur de rendu.

## RENDER-02 — Améliorer la netteté du rendu à tous les niveaux de zoom
**Priorité : P1**

**Problème observé :** le rendu est flou, particulièrement lors du zoom.

**Comportement attendu :** privilégier un rendu vectoriel SVG si le moteur et l'architecture le permettent. À défaut, utiliser un rendu raster dont la résolution s'adapte réellement au niveau de zoom et à la densité de pixels de l'écran.

**Points à examiner :**
- déterminer pourquoi le rendu actuel devient flou ;
- vérifier si une texture ou une image de résolution fixe est agrandie ;
- éviter une simple augmentation permanente de la résolution, qui pourrait provoquer une consommation mémoire excessive ;
- maintenir une netteté suffisante après zoom, dézoom et redimensionnement ;
- vérifier les performances et l'utilisation mémoire sur des documents longs.

Le choix SVG/raster doit reposer sur les capacités réelles du moteur et sur un compromis mesuré entre netteté, mémoire et fluidité.

---

# 11. Dépendances entre demandes

| Groupe | Éléments concernés | Raison |
|---|---|---|
| Intégrité des thèmes | THEME-01, THEME-EDITOR-04 | Les deux demandes concernent la corruption ou le remplacement de thèmes. |
| Thèmes et fenêtres | TOPBAR-11, THEME-01 | Le changement de thème doit rester cohérent et sûr dans toutes les fenêtres. |
| Bibliothèque | LEFT-06 à LEFT-09 | Présentation, comptage, métadonnées et filtres concernent le même ensemble de données. |
| Séparateurs | EDITOR-03, NORMS-10 | La syntaxe et le rendu du séparateur doivent être configurés ensemble. |
| Guillemets | NORMS-07 à NORMS-09 | Les conventions linguistiques et les niveaux d'imbrication doivent reposer sur le même système. |
| Onglets détachés | WINDOWS-03, WINDOWS-04 | L'aperçu et la zone de détachement forment une seule interaction. |
| Rendu et zoom | TOPBAR-08, RENDER-01, RENDER-02, FOCUS-02 | Les commandes, la netteté et le positionnement du rendu sont liés. |

# 12. Ordre de traitement recommandé

Cet ordre privilégie la sécurité des données, puis les bugs qui perturbent les interactions, avant les améliorations d'interface et les nouvelles fonctionnalités.

1. **Sécuriser les données et les sauvegardes :** THEME-01, THEME-EDITOR-04, LEFT-12, NORMS-06.
2. **Corriger les bugs d'interaction :** LEFT-05, TOPBAR-05, THEME-EDITOR-01, WINDOWS-01, WINDOWS-02, NORMS-01, NORMS-02, FIND-03.
3. **Fiabiliser l'éditeur et le rendu :** EDITOR-01, EDITOR-02, RENDER-02, RENDER-01.
4. **Revoir l'organisation de l'interface :** panneau latéral, barre supérieure, mode focus, éditeur de thèmes et fenêtre des normes.
5. **Ajouter les fonctions éditoriales et typographiques :** syntaxe, tableaux, petites capitales, guillemets, normes linguistiques et Quick Parts.
6. **Ajouter les fonctions d'import et de recherche avancée :** TOPBAR-03, FIND-01, FIND-02 et filtres de bibliothèque.

# 13. Livrables attendus de l'agent

Pour chaque lot de travail :

1. **Diagnostic :** cause identifiée, fichiers/composants concernés et éventuelles contraintes techniques.
2. **Plan :** modifications prévues, dépendances et risques de régression.
3. **Implémentation :** changements limités au périmètre convenu, sans refonte opportuniste d'autres parties.
4. **Vérification :** tests exécutés, cas limites contrôlés et résultats observés.
5. **Bilan :** exigences terminées, exigences partiellement réalisées, décisions restant à prendre et problèmes connus.

Ne pas déclarer une demande terminée sur la seule base d'une compilation réussie. Vérifier le comportement réel dans l'application, notamment les interactions multi-fenêtres, les états d'erreur et la conservation des données.
