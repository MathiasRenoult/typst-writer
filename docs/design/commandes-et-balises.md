# Commandes, balises et insertion rapide

> **Statut : implémenté dans l'application — catalogue, palette, formulaires
> simples, métadonnées et formes canoniques des balises inline et des blocs.
> Les balises ont des noms courts (`c`, `d`, `g`, `b`, `m`, `e`, `gr`, `t`,
> `s`, `bib`, `gs`, `idx`) ; les noms longs restent lus comme alias. Les
> panneaux complexes gardent leur état et leur panneau, mais les nouvelles
> insertions écrivent la forme canonique quand elle existe.**
> Ce document part de l'état actuel de `typst-writer` et propose une
> architecture pour unifier le langage, les commandes et l'insertion de
> fragments de texte.

## 1. Le problème réel

Le dépôt mélange aujourd'hui trois couches qui se ressemblent :

1. **Les messages iced** (`src/app/message.rs`) : un grand enum de transport,
   avec des variantes très précises (`ChartType`, `CalloutChamp`,
   `NoteTexte`, `BiblePanneauAjouter`, etc.).
2. **Les commandes utilisateur** : boutons de la barre d'outils, menus,
   raccourcis clavier, gestes du panneau. Ils sont souvent écrits à la main,
   chacun avec son libellé, son icône et son routage.
3. **Les balises du langage** : `{couleur:…}`, `{glo:…}`, `{date:…}`,
   `{{Jean 3:16}}`, `{chart …}`, `{callout …}`, `{encadré}`, `{glossaire}`,
   `{bibliographie}`, `[[TOC]]`, `{{index}}`, les lignes de propriétés…

Chaque balise possède son propre lecteur dans `parse.rs`, son propre
sérialiseur, son propre panneau et parfois ses propres alias. C'est cette
dispersion qui produit les « quatorze façons de lire une balise ».

Un `Message` n'est pas une commande : c'est un **effet** envoyé à iced. Il ne
porte ni nom stable, ni icône, ni description, ni disponibilité, ni schéma de
paramètres. Il ne doit donc pas devenir la surface publique des commandes.

## 2. Vocabulaire proposé

- **Commande** : intention utilisateur nommée, avec un identifiant stable,
  un libellé, une description, une icône, des alias, une portée, une
  disponibilité, un raccourci éventuel.
- **Balise** : commande particulière qui produit une construction du langage
  (`couleur`, `date`, `encadré`, `graphique`, `bible`, `sommaire`…).
- **Message** : effet iced existant, produit par l'exécuteur d'une commande.
- **Snippet** : commande composée qui insère un fragment plus large
  (plusieurs balises, du texte, des valeurs préremplies).
- **Contexte** : mode de vue, dialecte (`.tw` ou `.typ`), sélection, présence
  dans du code ou un littéral, plugins actifs, verrou du projet.
- **Formulaire de balise** : panneau généré depuis le schéma de la balise,
  pour la personnalisation complète.

## 3. Le modèle central

### 3.1 Catalogue de commandes

```rust
pub struct Commande {
    pub id: CommandeId,
    pub nom: &'static str,          // nom stable, sans accent
    pub titre: &'static str,        // libellé français
    pub description: &'static str,
    pub alias: &'static [&'static str],
    pub categorie: Categorie,
    pub icone: IconeId,
    pub raccourci: Option<Raccourci>,
    pub portee: Portee,
    pub action: ActionCommande,
}

pub enum ActionCommande {
    Message(fn(&Contexte) -> iced::Task<Message>),
    Balise(BaliseId),
    Panneau(PanneauId),
    Snippet(SnippetId),
}
```

`CommandeId` est un identifiant `Copy`, stable et sérialisable. Le catalogue
est une constante statique. Il ne contient pas de `Message` construit : les
messages peuvent porter des données de projet. Une fonction ou un exécuteur
central construit l'effet au moment de l'invocation.

L'exécuteur unique est la seule porte :

```text
palette / toolbar / clavier / `,,` / `/`
                 │
                 ▼
        CommandeId + Contexte
                 │
                 ▼
          executer(id, contexte)
             │           │
       Balise/Snippet   Message
             │           │
      insertion ou      routeur existant
      formulaire
```

Conséquences :

- la barre d'outils, les menus, la palette et les raccourcis lisent le même
  catalogue ;
- un bouton ne peut pas exister sans libellé, icône et description ;
- une commande indisponible est affichée avec sa raison (« aucune sélection »,
  « projet en lecture seule », « plugin désactivé ») au lieu de disparaître
  silencieusement ;
- les `Message` restent le transport interne, pas la documentation.

### 3.2 Catalogue de balises

Chaque balise décrit sa grammaire, ses valeurs par défaut et son panneau :

```rust
pub struct BaliseSpec {
    pub id: BaliseId,
    pub noms: &'static [&'static str], // nom canonique + alias
    pub placement: Placement,          // en ligne, bloc, les deux
    pub champs: &'static [ChampSpec],
    pub corps: CorpsSpec,              // aucun, bloc suivant, lignes, apparié…
    pub defaut: Option<Defaut>,        // insertion directe possible ?
    pub panneau: Option<PanneauId>,    // formulaire générique
    pub icone: IconeId,
    pub categorie: Categorie,
    pub description: &'static str,
    pub rendre: fn(&BaliseValeur) -> Result<String, ErreurBalise>,
}

pub struct ChampSpec {
    pub cle: &'static str,
    pub libelle: &'static str,
    pub type_: TypeChamp, // texte, mesure, nombre, booléen, énum, couleur, chemin, csv, inline…
    pub obligatoire: bool,
    pub defaut: Option<Valeur>,
    pub aide: &'static str,
    pub options: &'static [OptionValeur],
}
```

Le même schéma sert à :

- lire la balise (un seul analyseur d'arguments) ;
- écrire sa forme canonique ;
- générer son formulaire ;
- valider ses valeurs ;
- alimenter la recherche et l'aide ;
- produire les tests.

Une balise n'a plus le droit d'inventer son propre analyseur ni son propre
échappement. Les cas particuliers deviennent des **types de champ** ou des
**types de corps**, pas des branches nouvelles dans le parseur.

## 4. Syntaxe unifiée

### 4.1 Principe

Le langage garde deux niveaux distincts :

1. **Le Markdown de prose** : titres, paragraphes, emphases, listes,
   citations, liens, notes de bas de page, code, formules. C'est la langue
   d'écriture quotidienne, elle reste stable.
2. **Les directives d'application** : toutes les balises spécifiques au
   projet, les plugins et les blocs structurés. Elles utilisent **une seule
   enveloppe**.

Le double crochet est proposé pour les directives :

```text
{{nom [champs positionnels] [clé=valeur …] [| contenu]}}
```

Pourquoi `{{…}}` :

- c'est une enveloppe visible, impossible à confondre avec une emphase ou un
  titre ;
- elle est déjà présente dans le langage pour la Bible, qu'on peut absorber
  comme balise `bible` ;
- elle permet d'écrire un contenu inline dans la même construction ;
- elle distingue les directives des accolades Typst et des lignes de
  propriétés existantes.

Le `,,` proposé par l'auteur reste un **déclencheur d'interface**, pas la
syntaxe canonique du document. Le menu insère `{{…}}` ; l'utilisateur n'a
jamais besoin de connaître l'enveloppe pour l'utiliser.

### 4.2 Forme générale

- Nom : une à trois lettres pour les balises courantes (`c`, `d`, `g`, `b`,
  `m`, `e`, `gr`, `t`), ou un nom long quand il est plus clair (`bib`,
  `gs`, `idx`) ; les paramètres gardent des noms lisibles (`type`, `titre`,
  `couleur`).
- Champs positionnels : déclarés explicitement par la balise, dans l'ordre.
- Champs nommés : `clé=valeur`, valeur entre guillemets si elle contient un
  espace.
- Contenu inline : après `|`, jusqu'au `}}` fermant ; il est analysé comme
  une suite inline et peut contenir les autres balises.
- Corps de bloc : le schéma déclare le mode de corps. Les lignes suivantes
  appartiennent à la balise jusqu'à une ligne vide, un bloc structurel, ou
  un marqueur de fermeture explicite.
- Échappement : `\{{` et `\|` restent littéraux ; les règles d'échappement
  sont les mêmes que celles du langage.

### 4.3 Exemples

```text
{{c red|texte important}}
{{g justification}}
{{d 2026-11-16}}
{{b "Jean 3:16"}}
{{m auteur}}

{{e type=note titre="À retenir"|Le texte encadré.}}

{{gr type=bar titre="Ventes"}}
region | ventes
Nord | 30
Sud | 70

{{ch type=pie titre="Répartition"}}
region,ventes
Nord,30
Sud,70

{{t}}
| A | B |
| --- | --- |
| 1 | 2 |

{{s}}
{{bib}}
{{gs}}
{{idx}}
{{p augustin}}
{{p augustin|complet}}
{{p augustin|nom+dates}}
{{ip}}
```

Les noms canoniques sont courts : `c` (couleur), `d` (date), `g`
(glossaire), `b` (bible), `m` (métadonnée), `e` (encadré), `gr`
(graphique simple), `ch` (graphique primaviz), `t` (tableau), `s`
(sommaire), `bib` (bibliographie), `gs` (section de glossaire), `idx`
(index biblique), `p` (personne citée), `ip` (index des personnes). Les noms
longs (`couleur`, `date`, `glossaire`, `bible`,
`meta`, `encadre`, `graphique`, `tableau`, `sommaire`, `bibliographie`,
`glossaire-section`, `index-biblique`, `personne`, `index-personnes`,
`index-nom`) restent
lus comme alias, ainsi que les anciennes formes (`{couleur:…}`, `{glo:…}`,
`{chart}`, `{callout}`, `{encadré}`, `[[TOC]]`, `{{index}}`,
`{{Jean…}}`).

Une personne ne retient que sa **clé** dans la balise. Sans expression, les
formes de première mention et de reprise des normes s'appliquent (nom simple
par défaut) ; `|simple`, `|complet` ou `|alias` choisissent une forme nommée ;
une liste de champs séparés par `+` (`nom+dates`, `simple+naissance+mort`,
`nom_complet+dates`) choisit précisément ce qui est affiché ; `|auto` écrit la
règle des normes explicitement ; un autre texte reste une forme libre. Le
panneau d'insertion de la palette filtre le registre et remplit la fiche.

`{{m clé}}` est la balise des métadonnées du projet : elle lit la valeur
de `metadonnees.toml` par la même voie que les quick parts de l'en-tête.
La palette expose chaque métadonnée existante comme une entrée
« Métadonnée · clé », de sorte qu'aucune clé n'ait à être apprise par cœur.

### 4.4 Ce qui reste du Markdown

Il ne faut pas unifier `**gras**`, `*italique*`, `[lien](…)` et `^[note]`
avec les directives : ce sont des primitives de prose, pas des plugins. En
revanche, les commandes d'insertion peuvent les produire, et le catalogue
peut les décrire comme des **snippets de prose**.

Le cas de la note de bas de page est révélateur : c'est du Markdown, mais
son contenu accepte toutes les balises inline. Le parseur récursif actuel
fait déjà cela ; le panneau de note doit simplement arrêter d'échapper le
contenu.

### 4.5 Compatibilité et migration

Le langage existant est gelé par `langage-intermediaire.md`. Une nouvelle
syntaxe ne doit pas réécrire silencieusement les documents de l'auteur.

- Le parseur accepte la forme canonique et toutes les formes historiques.
- L'AST conserve l'**origine de syntaxe** d'une balise (canonique ou
  historique) pour que la sérialisation d'un document ancien reste fidèle.
- Une nouvelle insertion utilise toujours la forme canonique.
- Une commande explicite « Normaliser les balises » propose un diff, une
  sauvegarde et une écriture atomique. Elle n'est jamais automatique.
- Un diagnostic non bloquant signale les formes historiques et propose la
  normalisation ligne par ligne.
- Une balise inconnue n'est plus interprétée comme une couleur : elle reste
  du texte et reçoit un diagnostic si elle ressemble à une directive.

## 5. Le menu de commandes

### 5.1 Une seule palette, deux modes

La palette est un composant unique avec deux entrées :

- **mode balises** (`,,` puis une lettre) : uniquement les balises et
  snippets insérables dans le contexte courant ;
- **mode système** (`/` puis du texte, ou `Ctrl+K` partout) : toutes les
  commandes disponibles, y compris les actions qui ne sont pas des balises
  (enregistrer, exporter, changer de thème, ouvrir un panneau…).

Les deux modes partagent la recherche, les raccourcis, les icônes, les
catégories et la navigation. Seul le filtre de disponibilité et l'action par
défaut changent.

### 5.2 Déclencheurs

- `,,` ouvre le menu s'il est au début d'un mot (précédé d'un blanc ou du
  début de ligne), suivi d'une lettre, hors code et hors littéral.
- `/` ouvre le menu système au début de ligne ou après un blanc, suivi d'une
  lettre. Un `/` collé à un mot (`et/ou`, URL) ne déclenche rien.
- `Ctrl+K` ouvre le menu système partout ; c'est le chemin de secours si un
  déclencheur est ambigu.
- `Échap` ferme le menu sans toucher au texte déjà saisi.
- Une sélection non vide et `,,` proposent d'abord les balises qui enveloppent
  la sélection (`gras`, `couleur`, `encadré`…).

Le texte reste dans l'éditeur ; le menu est un recouvrement non modal. Les
caractères continuent d'alimenter la requête, les flèches déplacent la
sélection, `Entrée` exécute, `Tabulation` ouvre la configuration.

### 5.3 Contrat clavier

| Touche | Effet |
|---|---|
| `Entrée` | insérer avec les valeurs par défaut, ou ouvrir le formulaire si la balise a des champs obligatoires sans défaut |
| `Tabulation` | ouvrir la configuration de la balise sélectionnée |
| `Maj+Tabulation` | revenir au champ précédent dans un formulaire |
| `↑` / `↓` | parcourir les résultats |
| `Échap` | fermer sans insérer |
| `Ctrl+Entrée` | forcer l'ouverture du formulaire |
| `Ctrl+K` | ouvrir la palette système |

### 5.4 Contenu de la palette

À vide, la palette montre :

- les commandes récentes et favorites du contexte ;
- les actions rapides pertinentes (selon la sélection, le mode et les plugins) ;
- les catégories.

Une recherche non vide filtre sur le titre, les alias, la description, le nom
canonique et les mots-clés. Le pliage des accents et de la casse est
obligatoire. Chaque ligne montre l'icône, le libellé, la catégorie et le
raccourci éventuel. Une commande indisponible est grisée avec sa raison.

## 6. Insertion par défaut et formulaire

Chaque balise a deux chemins :

1. **Insertion par défaut** : des valeurs sûres, issues du schéma, des normes
   du document ou du contexte (sélection, dernier choix, chapitre courant).
2. **Formulaire complet** : un panneau généré par le schéma, navigable au
   clavier, avec validation et aperçu.

Le formulaire ne doit pas être un panneau écrit à la main par balise. Les
types de champ couvrent les besoins :

- texte, texte long, nombre, mesure, booléen, énumération ;
- couleur, chemin de fichier, citation, référence biblique ;
- CSV pour un graphique ;
- contenu inline ou bloc pour un encadré, une figure, une note.

Les widgets spécialisés (sélecteur de fichier, composeur de citation,
recherche biblique) restent des **rendus de champ** branchés sur le schéma,
pas des panneaux concurrents.

Le formulaire affiche :

- un aperçu de la syntaxe canonique qui sera insérée ;
- une validation en direct (champ obligatoire, valeur invalide) ;
- un bouton « Insérer », « Réinitialiser », « Insérer par défaut » ;
- le curseur final, à l'endroit utile (dans le corps, dans le titre…).

L'insertion est une seule action annulable : le texte inséré et le placement
du curseur ne créent pas deux étapes d'historique.

## 7. Ce que cela change dans le code

### 7.1 Modules proposés

```text
src/commandes/
  mod.rs          CommandeId, Contexte, exécuteur
  catalogue.rs    catalogue statique des commandes
  recherche.rs    index de recherche et classement
  insertion.rs    insertion canonique .tw/.typ, sélection, curseur
  snippets.rs     commandes composées

src/langage/balises/
  mod.rs          BaliseId, BaliseSpec, registre
  grammaire.rs    un seul analyseur d'enveloppe et d'arguments
  schema.rs       champs, corps, validation
  serialisation.rs forme canonique et formes historiques

src/ui/
  palette.rs      palette unique, modes balises/système
  formulaire_balise.rs formulaire généré
```

Les routeurs existants restent les exécutants des messages. On ne remplace
pas `Message` par des chaînes de caractères.

### 7.2 Migration progressive

**Phase 0 — Inventaire.**
Lister toutes les commandes utilisateur, tous les raccourcis, toutes les
balises et leurs alias. Écrire les schémas avant de toucher au parseur.

**Phase 1 — Catalogue et palette système.**
Extraire le catalogue, brancher la barre d'outils, `plan_touche` et un premier
`/` / `Ctrl+K` sur les messages existants. Aucun changement de syntaxe. Cette
phase rend déjà toutes les actions cherchables.

**Phase 2 — Menu `,,`.**
Utiliser les insertions existantes (`NoteInserer`, `ChartOuvrir`,
`CalloutOuvrir`, `BiblePanneau`, `GlossaireInserer`…) derrière le catalogue.
Les balises sans panneau s'insèrent directement ; les autres ouvrent leur
panneau.

**Phase 3 — Grammaire unifiée.**
Introduire `BaliseSpec` et l'analyseur unique, avec adaptateurs historiques.
Les nouvelles insertions produisent la forme canonique. Les documents
existants restent lisibles et ne sont pas réécrits.

**Phase 4 — Formulaires générés.**
Migrer d'abord les balises simples (`couleur`, `glossaire`, `date`), puis
`graphique` et `encadre`. Les panneaux complexes deviennent des champs
spécialisés du formulaire.

**Phase 5 — Snippets, normalisation et finitions.**
Commandes composées, favoris, historique, migration assistée, aide générée.

## 8. Bonifications

1. **Snippets et macros.** Une commande peut insérer plusieurs balises et
   textes avec des variables (`titre`, `auteur`, `date`, `sélection`). Exemple :
   `,,these` insère un titre, un encadré et une bibliographie. Les snippets
   sont stockés par projet ou globalement et apparaissent dans `,,` et `/`.

2. **Commandes contextuelles.** Avec une sélection, `,,` propose d'abord
   l'enveloppement ; sans sélection, il propose l'insertion. Dans une note,
   il propose les balises compatibles avec une note. Dans un bloc de code, il
   se ferme.

3. **Aperçu et validation avant insertion.** Le menu affiche la syntaxe
   canonique et, si possible, un aperçu Typst. Une balise invalide n'est
   jamais insérée silencieusement.

4. **Historique, favoris et fréquence.** Les commandes récentes et favorites
   apparaissent en tête ; l'utilisateur peut épingler ses snippets. Le
   classement reste local au poste.

5. **Mode débutant / expert.** Le mode débutant montre les formulaires et les
   valeurs par défaut ; le mode expert montre la syntaxe, les champs avancés
   et permet l'édition directe.

6. **Documentation générée.** Le catalogue produit les libellés, icônes,
   raccourcis, aides et la page F1. Une commande ou une balise sans métadonnée
   fait échouer une épreuve de complétude.

7. **API de plugin.** Un plugin enregistre un `BaliseSpec`, un rendu et une
   icône. Il n'ajoute pas de branche dans `parse.rs` ni de bouton dans la
   barre d'outils.

8. **Normalisation assistée.** Une commande montre le diff entre formes
   historiques et canoniques, avec sauvegarde préalable. Elle ne s'exécute
   jamais à l'ouverture ni à la sauvegarde.

9. **Répétition de commande.** Une touche ou une entrée de palette rejoue la
   dernière commande avec les mêmes paramètres, utile pour les insertions
   répétées.

10. **Recherche par intention.** Les alias décrivent l'intention
    (« insérer une note », « encadrer », « citer un livre »), pas seulement
    le nom technique de la balise.

## 9. Épreuves à prévoir

- Toute commande visible dans la barre d'outils, un menu ou un raccourci a
  une entrée de catalogue, un libellé, une icône et une portée.
- Toute balise a un `BaliseSpec`, un nom canonique, des alias historiques,
  des valeurs par défaut ou un formulaire.
- `parse(serialiser(document)) == document` pour les documents canoniques,
  les documents historiques et les documents mixtes.
- Un document historique n'est pas modifié par une simple ouverture ou
  sauvegarde.
- Insertion par défaut : le texte produit se relit comme la même balise.
- Formulaire : chaque champ se valide, le curseur final est correct, une
  annulation ne laisse rien.
- Clavier : `,,`, `/`, `Ctrl+K`, flèches, `Entrée`, `Tab`, `Échap`,
  `Ctrl+Entrée` dans un éditeur, une note et un aperçu.
- Les balises s'imbriquent dans une note et dans une autre balise.
- Une balise inconnue ne devient jamais une couleur ou une commande Typst.

## 10. Décisions à prendre

1. **Compatibilité** : conserver les documents `.tw` existants (recommandé)
   ou repartir sur un langage vert, sans adaptateurs.
2. **Enveloppe** : `{{…}}` (recommandée) ou une autre enveloppe unique
   (`::`, `…`).
3. **Noms canoniques** : français (`couleur`, `glossaire`, `encadre`) avec
   alias anglais, ou l'inverse.
4. **Corps des blocs** : « jusqu'à la ligne vide », « bloc suivant », ou
   balises appariées `{{nom}}…{{/nom}}`.
5. **Périmètre initial** : catalogue et palette d'abord (livraison rapide),
   ou grammaire unifiée d'abord (plus propre, plus long).
