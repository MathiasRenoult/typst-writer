# Les normes

La fenêtre des normes — `Ctrl+P`, ou le bouton de la barre — tient trois
onglets : **Mise en page**, **Typographie**, **Décor**. Ce qu'elle montre
est une copie de travail : seul « Appliquer » touche le projet, et les
valeurs fautives sont dites champ par champ avant que quoi que ce soit ne
soit écrit.

## Mise en page

Le papier (format, orientation), les marges, les numéros de page, le
texte courant (police, taille, interligne, retrait, justification,
langue), les titres de chapitres (police, tailles, gras, casse,
numérotation) et les couleurs.

Les choix finis sont des **listes déroulantes** ; chaque paramètre porte
son **infobulle** — laisser la souris dit ce qu'il règle et sous quelle
forme l'écrire. Toute mesure Typst est acceptée : `2cm`, `1in`, `24pt`.

## Typographie

La composition du texte réglée **par langue** : un défaut — `française`,
`anglaise`, `aucune` — puis les réglages fins, un par un.

En français : une espace insécable devant `; : ! ?` et les guillemets,
normale après ; en anglais : la ponctuation colle. En toute langue, la
virgule et le point ne s'ouvrent jamais — `3.14` tient. Les guillemets se
choisissent par niveau d'imbrication : `«»`, `""`, `''`.

Le code inline, les blocs bruts et les caractères échappés ne reçoivent
aucune règle.

## Décor

Les mesures que la source ne porte jamais : **couleur de bordure,
épaisseur, rayon des coins, marge intérieure, remplissage**. C'est là
qu'un `{encadré}` prend sa tenue ; changer une valeur ici change le
document entier, sans toucher à une ligne.

Voir [Écrire](ecrire.md).
