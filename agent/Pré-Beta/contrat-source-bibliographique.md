# Contrat de source bibliographique partagé

*R3 — références locales correctes. Ce contrat est **écrit** et vérifié par
des tests ; chaque règle nomme l'épreuve qui la garantit.*

## La source

La bibliothèque d'un projet est l'ensemble des notices lues dans les
fichiers `.bib` **cités par le document** :

- la bibliographie conventionnelle du projet (`references.bib`, puis
  l'export Zotero — `project::find_bibliography`) ;
- les bibliographies nommées par les directives `#bibliography("…")` de
  l'assemblage ;
- celles nommées par les directives des **chapitres inclus**, dont le
  chemin se résout à la place du fichier qui le porte.

Chaque notice est une `project::BibliographyEntry` : clé, genre, auteurs,
titre, année. Il n'existe **pas de seconde lecture** : qui montre, cite,
complète ou compile lit cette source.

## Les consommateurs

| Qui | Lit | Où |
| --- | --- | --- |
| Bibliothèque visible (panneau) | `state.bibliography` | `ui/sidebar.rs` |
| Recherche (auteur/titre/année/clé) | la même `BibliographyEntry` | `ui/sidebar.rs`, `app/composeur.rs` |
| Complétion et composeur | `state.bibliography` | `app/assistance.rs` |
| Citations, neutralisation, diagnostics | `cles_vivantes` | `langage/pipeline.rs` |
| Aperçu et PDF | la **même** unité de compilation | `State::compilation_unit_avec_diagnostics` → `typst::render` |

L'aperçu et le PDF partagent la même unité de compilation
(`State::compilation_unit_avec_diagnostics`, appelée par la compilation
d'aperçu et par l'export) : ce que l'aperçu montre est ce que le PDF
contient. Les exports tardifs d'une vue remplacée n'écrivent rien
(*les_exports_tardifs_dune_vue_remplacee_necrivent_pas*).

## Les invariants

1. **Une clé vivante est une vraie référence.** Présente dans au moins
   une bibliographie citée du projet — où qu'elle soit citée — elle reste
   `@clé` ou `#cite(<clé>)` jusqu'au PDF. Jamais neutralisée à tort
   (*une_cle_non_ascii_vivante_reste_une_vraie_reference*,
   *deux_bibliographies_a_cles_communes_ne_se_contredisent_pas*).

2. **Une clé inconnue devient un texte visible et un diagnostic.**
   `[clé]` dans le document — jamais une fausse référence — avec un
   avertissement portant la clé, la ligne et le fichier
   (*une_citation_inconnue_est_signalee_et_ne_devient_jamais_une_reference*,
   *un_cite_inconnu_est_signale*,
   *une_citation_perdue_dans_un_chapitre_dit_son_fichier*). Le silence
   est un défaut.

3. **Une collision se dit.** Une même clé dans deux bibliographies rend
   la citation ambiguë : un diagnostic nomme la clé et les deux
   fichiers (*deux_bibliographies_a_cles_communes_ne_se_contredisent_pas*).

4. **Les clés non ASCII sont des clés.** Même alphabet que le parseur ;
   une clé non ASCII manquante suit la règle 2 au lieu de casser la
   compilation (*une_cle_non_ascii_manquante_ne_casse_jamais_le_document*).

5. **Une adresse électronique n'est jamais une clé.** Un `@` suivi d'un
   point interne s'échappe (`\@`) et se relit intact au PDF
   (*une_adresse_electronique_nest_jamais_une_cle*).

6. **La source citée est celle du PDF.** Le texte extrait du PDF réel
   porte les auteurs et le titre de la notice citée, y compris avec des
   auteurs multiples (*le_texte_extrait_du_pdf_porte_la_source_citee*,
   *les_auteurs_multiples_se_lisent_dans_le_texte_extrait*).

7. **Une compilation en erreur n'avale rien.** Les erreurs du document
   se disent avec son message ; les avertissements de préparation
   restent visibles à côté, et le dernier aperçu valide est déclaré
   ancien (*une_compilation_en_erreur_dit_erreurs_et_avertissements*,
   *un_apercu_affiche_sous_des_erreurs_est_indique_ancien*).

## Ce que ce contrat ne couvre pas encore

- L'import d'un `.bib` local (choix, fusion) : R3 l'annonce, il reste à
  faire. L'édition de notices, elle, préserve les champs non modifiés
  (*modifier_une_reference_ne_reecrit_que_ses_champs*) et laisse aux
  `.bib` ce qu'elle ne comprend pas
  (*ecrire_une_reference_laisse_un_bib_que_typst_accepte*).
- Un `@` dans l'URL d'un lien (`https://…/@user`) : traité avec le
  chantier des liens.
- Zotero en écriture : hors périmètre de R3 (lecture seule en R4).
