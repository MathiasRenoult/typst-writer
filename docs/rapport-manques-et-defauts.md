# Rapport des manques et défauts

État de l'application au 27 octobre 2026, après le jalon 4 (premier pas) et
avant la réparation de ce qui suit.

**Méthode.** Quatre audits indépendants ont passé le dépôt en revue —
interface, langage, rendu/LSP, projet et persistance — puis les constats
les plus graves ont été **revérifiés à la source** avant d'entrer ici. Ce
qui est marqué *(vérifié)* a été reproduit ou lu dans le code par nos
soins ; le reste est un constat d'audit, sérieux mais non reproduit.

**Réparé pendant l'audit** (trois reliquats, signalés puis corrigés) :
un test qui lisait vos données réelles (`~/.typst-writer/.../mise-en-page.toml`)
sans être ignorable ; le décodage HTTP « morcelé » qui coupait par octets
là où les tailles sont en caractères (un accent corrompait la réponse
Zotero) ; la pagination de l'export BibTeX qui comptait des octets au lieu
d'entrées.

---

## A. Données : ce qui peut perdre du texte

1. **[Majeur] L'auto-sauvegarde s'arrête dès que le document ne compile pas.**
   `src/app/mod.rs:906-908` ne pose `save_pending` que si la compilation a
   réussi ; `SaveTick` n'écrit que sur ce drapeau. Un document en erreur
   Typst — un bloc brut mal fermé, un `#include` manquant — n'est donc
   **jamais** écrit, et une coupure emporte tout depuis le dernier
   Ctrl+S explicite. *(vérifié)*

2. **[Majeur] Vider un champ de notice corrompait l'entrée BibTeX.**
   `retirer_champ` (`src/project/mod.rs`) coupait la valeur mais laissait
   l'accolade fermante : vider « année » refermait l'entrée et expulsait
   le titre. Déclenché par le formulaire de référence dès qu'un champ
   facultatif présent est laissé vide. *(vérifié — la reproduction exacte
   est dans l'audit, le mécanisme lu dans le code)*

3. **[Majeur] `document.tw` n'est pas écrit atomiquement.**
   `src/editor/mod.rs:679` écrit directement, alors que les migrations du
   même fichier passent par temporaire+renommage : une coupure au milieu
   tronque la source d'écriture. Les autres fichiers de persistance
   (session, normes, `zotero.json`, `references.bib`, `main.typ`) sont
   dans le même cas.

4. **[Majeur] Les échecs d'écriture sont silencieux.** Sauvegarde
   (`app/mod.rs:386`), enregistrement de référence (`app/mod.rs:586` — le
   dialogue se ferme même en échec, la saisie est jetée), « Enregistrer
   sous » (`fichiers.rs:476`) : tout part en `eprintln`, la barre d'état
   reste sur « Non enregistré » sans dire pourquoi.

5. **[Moyen] Fichiers corrompus : repli silencieux.** `zotero.json`
   corrompu → la liaison disparaît sans mot ; `mise-en-page.toml` corrompu
   → défauts, puis un « Appliquer » **écrase** le fichier ; `session.json`
   corrompu → session vide. La sérialisation en échec écrit même un
   fichier **vide** (`mise_en_page/modele.rs:362`, `zotero/keys.rs:38`),
   qui détruit la valeur précédente.

6. **[Moyen] Suppressions sans confirmation.** Le chapitre part au clic
   sur la corbeille (`fichiers.rs:255`, résultat dans `let _ =`), un thème
   part du disque au clic (`themes.rs:128`), la liaison Zotero se défait
   en un clic. Aucun dialogue, aucune annulation.

7. **[Mineur] Aucun verrou d'instance.** Deux instances ouvertes sur le
   même projet s'écrasent leurs auto-sauvegardes.

## B. Langage et pipeline `.tw`

8. **[Majeur] Les couleurs `#ff0000` sont documentées… mais ne compilent
   pas.** Le générateur injecte le nom brut dans `#text(fill: …)` ;
   Typst 0.15 refuse `#` en code. Vérifié au compilateur. Aucun test de
   compilation ne couvre les couleurs hex. *(vérifié — reproduit avec la
   CLI Typst)*

9. **[Majeur] Le contrat de round-trip est faux sur les bords blancs.**
   `est_canonique` accepte des contenus à blancs initiaux/finaux que
   `parse` rogne : `Paragraphe([Texte(" x")])` est déclaré canonique et ne
   se relit pas, et `Paragraphe([Texte("x"), Litteral(" ")])` y **perd un
   caractère**. Le générateur de tests au hasard ne produit jamais ces
   formes, d'où leur survie. *(vérifié — contre-exemples lus dans le code)*

10. **[Majeur] La typographie perd son contexte aux frontières d'éléments.**
    `appliquer` travaille sur chaque `Texte` isolément : l'insécable avant
    `;` disparaît si le signe ouvre un élément, l'espace après `.` n'est
    pas posée en fin d'élément, et le niveau des guillemets se réinitialise
    à chaque élément (« `«` dans un texte, `«` dans un gras » donnent deux
    fois le niveau 1).

11. **[Moyen] Le rempart de tête de ligne Typst ne couvre pas le cas « après
    un blanc ».** `echapper_texte` ne protège `- + = /` qu'en position 0 ;
    un paragraphe commençant par un littéral espace génère ` - x`, que
    Typst rend en liste. Vérifié au compilateur.

12. **[Moyen] La migration change la structure en silence.** Une ligne
    Typst « 1. Introduction » devient une liste numérotée `.tw` ; un item
    indenté devient un item de premier niveau ; le rapport de migration,
    pourtant promis « jamais en silence », est jeté dans le chemin de
    production (`project/model.rs:249`).

13. **[Moyen] Les glyphes de citation peuvent diverger du parseur.**
    `marques.rs` lit les appels avec un jeu de clés différent de
    `parse.rs` (`: . +` en plus) et une profondeur de parenthèses fausse :
    les glyphes peuvent manquer ou couper là où la note existe.

14. **[Faible]** CRLF : un fichier ouvert puis sauvegardé est reformaté en
    LF, et les plages d'octets de la couche de blocs comptent mal les
    `\r`. `_` intramot devient de l'italique (`snake_case` est détruit).
    Les valeurs de typographie inconnues tombent en `aucune` sans dire.
    `titres_source` s'apparie par `zip`, fragile si le texte a bougé.

## C. Rendu, aperçu, LSP

15. **[Moyen] Le plafond de rasterisations est remis à zéro à chaque
    invalidation** : un redimensionnement continu relance des rafales de
    tâches dont les précédentes tournent encore — pointes mémoire non
    bornées. La limite existe, elle n'est pas tenue dans le temps.

16. **[Moyen] Un document tenu dans la fenêtre n'affiche que sa première
    page** : `iced` ne signale plus de viewport, `visible` reste figé sur
    `(0, 0)`, et les pages 2+ n'apparaissent qu'après un défilement
    manuel. Idem après un changement de chapitre.

17. **[Moyen] Échec de compilation : les anciennes textures ne sont pas
    libérées** et l'aperçu affiche « Aucune page » au lieu de garder la
    dernière image sous un bandeau — la promesse du double tampon ne
    couvre pas l'erreur.

18. **[Moyen] LSP : `awaiting` peut bloquer indéfiniment** (aucun timeout,
    aucune annulation) ; une panne de tinymist **désactive** l'assistance
    sans reprise ni message ; un cadre malformé tue la session.

19. **[Moyen] Les diagnostics se décalent.** Au changement de document, les
    anciens restent affichés ; les positions sont recalculées sur le texte
    courant, pas sur celui que le serveur a vu — les sauts pointent
    ailleurs que le problème.

20. **[Faible]** Les `.typ` du projet sont relus entièrement à chaque
    compilation ; trois copies du texte vivent en mémoire ; la couleur du
    papier se lit sur le premier pixel de la première page ; l'échelle
    minimale de 1,5 px/pt double le coût sur écran 1,0.

## D. Interface et ergonomie

21. **[Majeur] Tous les retours passent par le panneau d'aperçu.** La
    barre d'état, les erreurs de titre vide, l'export, les thèmes :
    ~25 sites alimentent `export_status`, rendu **dans l'aperçu**. Masqué
    ou en mode focus, plus aucun retour n'est visible — l'application
    parle à un endroit qu'on ne regarde pas.

22. **[Majeur] La barre d'outils déborde à la taille de fenêtre par
    défaut** (≈1300 px de boutons pour 1180 px de fenêtre) : les boutons de
    droite sont rognés, sans repli. La fenêtre « Normes » (760×640) déborde
    d'une fenêtre ramenée à son minimum (540) : le pied avec « Appliquer »
    est hors écran.

23. **[Majeur] Aucune recherche ni remplacement.** Pour une thèse de
    plusieurs centaines de pages, rien ne permet de retrouver un mot.

24. **[Majeur] Le renommage de chapitre n'a pas le focus** : après le clic
    sur le crayon, la frappe part dans l'éditeur et s'insère dans le
    document. De même, la fiche de référence donne le même identifiant à
    deux champs, et une notice Zotero « lecture seule » a des champs
    éditables dont les modifications sont jetées.

25. **[Moyen] Culs-de-sac.** « Renommer le symbole » ne fait rien sans LSP ;
    les diagnostics d'autres fichiers n'ouvrent rien ; sans `references.bib`,
    plus aucune voie de créer la première notice ; les dialogues ouverts en
    mode focus disparaissent sans être fermés.

26. **[Moyen] Clavier : rien n'est documenté** (Ctrl+S/N/O/E/I/T, F12,
    Ctrl+Espace, Ctrl+Maj+Haut/Bas…), et Échap ferme certaines fenêtres
    pas d'autres — là où les pieds de page promettent « Échap : fermer ».

27. **[Faible]** Compte de mots affiché trois fois, nom de fichier deux ;
    la position de l'aperçu se choisit en cyclant (l'infobulle annonce
    l'état, pas l'effet) ; « Enregistrer sous » propose `.typ` et bascule
    l'éditeur dessus, éteignant la mise en forme ; la disposition n'est pas
    persistée ; deux libellés pour « Fermer »/« Annuler » ; icônes sans
    nom accessible ; titre de fenêtre statique.

## E. Projet, Zotero, thèmes

28. **[Majeur] Les deux promesses de migration ne tiennent pas.** La
    bibliothèque errante n'est jamais adoptée (le test « modèle intact »
    est toujours faux, car le modèle contient `@` dans ses commentaires) et
    la « reprise au prochain démarrage » n'a jamais lieu (le garde sort
    dès qu'un projet existe). Aucun test ne couvre cette migration.

29. **[Moyen] Zotero : l'export est figé au moment de la liaison** — aucune
    resynchronisation quand la collection bouge ; la promesse de clés
    stables (`citations.json`) n'est pas câblée (module mort) ; la
    découverte du dossier s'arrête au premier profil muet ; un `.bib`
    périmé peut rester la bibliographie après « Délier » (le premier `.bib`
    du dossier est pris au hasard de `read_dir`).

30. **[Moyen] Thèmes : l'import écrase un homonyme sans confirmation**, la
    suppression vise `{slug(nom)}.toml` (fragile si le nom a été édité), et
    deux thèmes homonymes rendent la sélection ambiguë.

31. **[Faible]** Création de projet : nom imposé (`projet`, `projet-2`…),
    pas de renommage ni de suppression ; un dossier existant est réutilisé
    silencieusement ; « aller à la définition » ignore `document.tw` ; la
    session ne rouvre que `document.tw`.

## F. Documentation

32. `zotero/mod.rs` promet `citations.json` (non câblé, cf. 29) ; le README
    omet `mise-en-page.toml`, `zotero.json`, `references-zotero.bib` et
    décrit mal Ctrl+O ; quatre commentaires de code sont périmés ; la doc
    du langage omet `Litteral`, les appels groupés et compte « dix règles »
    pour onze.

## G. Tests

33. **[Majeur] Un test lit vos données réelles** (corrigé pendant l'audit) ;
    **un test écrit dans votre dossier de thèmes** (`theming/mod.rs:1428`,
    `write_example()` cible `~/.typst-writer/themes/`) : `cargo test` y
    crée et supprime un fichier.

34. **[Moyen] Zones sans tests** : la migration à plat (d'où les défauts
    28), le retrait d'un champ BibTeX (d'où le 2), `fichiers.rs` entier
    (suppression, import, ouverture), l'import/suppression de thèmes, les
    fichiers corrompus.

35. **[Moyen] Le contrat de round-trip n'est pas éprouvé dans son sens
    vrai** : la propriété teste le générateur, jamais
    `est_canonique ⇒ round-trip` — d'où le défaut 9.

---

## Ordre de réparation proposé

1. **Ce qui perd du texte** : sauvegarde indépendante de la compilation
   (A1), écritures atomiques (A3), erreurs d'écriture montrées (A4),
   corruption BibTeX (A2 — *vérifier si réparé depuis*).
2. **Ce qui casse le langage** : couleurs hex (B8), round-trip sur les
   bords blancs (B9), typographie aux frontières d'éléments (B10).
3. **Ce qui piège au quotidien** : canalisation unique des retours (D21),
   débordement de la barre d'outils (D22), recherche (D23), focus du
   renommage (D24).
4. **Ce qui fait mentir une promesse** : migration (E28), clés Zotero
   stables (E29), resynchronisation Zotero (E29).
5. **Le reste**, par famille, en fin de course.

*Ce rapport est un état des lieux, pas un engagement de calendrier. Chaque
élément y est réparable ; les neuf majeurs le sont sans refonte.*
