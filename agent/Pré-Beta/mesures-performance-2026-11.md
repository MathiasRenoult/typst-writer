# Mesures de stabilité et performance — R8

Date : 2026-11-02. Objectif (feuille de route R8, livrable 3) : mesurer la
stabilité et les performances sur long document et bibliothèque
volumineuse, mémoire et rendu incrémental, et n'améliorer que les goulets
**prouvés**.

## Procédure reproductible

Deux bancs `#[ignore]` dans `src/typst/render.rs`, en release :

    cargo test --release -- --ignored bench_full_document --nocapture
    cargo test --release -- --ignored bench_bibliotheque_et_stabilite --nocapture

## Matériel

- Linux 7.2.8-9-omarchy x86_64
- rustc 1.97.1, typst-writer 0.3.0-alpha.3, build release (`strip = debuginfo`,
  dépendances en opt-level 2)

## Mesures

### Long document (35 085 caractères, 9 pages) — `bench_full_document`

| Mesure                       | Valeur    |
|------------------------------|-----------|
| Compilation complète         | 31,46 ms  |
| Rasterisation (pages visibles) | ~11 ms/page (banc frère : 3,23 ms pour la page d'une bibliographie chargée) |
| Compilation après une frappe | 21,71 ms  |
| Taille d'une page            | 595 × 842 pt |

### Long document .tw + bibliothèque volumineuse — `bench_bibliotheque_et_stabilite`

| Mesure                          | Valeur        |
|---------------------------------|---------------|
| Parse .tw long (30 160 o, 600 blocs) | 1,83 ms  |
| Sérialisation                   | 197 µs        |
| **Stabilité** : `parse(serialiser(parse(c))) == parse(c)` | **tenue** (600 blocs) |
| Parse bibliographie 1 000 notices | 1,21 ms     |
| Compilation Typst **avec** 1 000 notices | 81,70 ms |
| Rasterisation page 0            | 3,23 ms       |
| Mémoire (pic processus, VmHWM)  | 98 848 kB (~96 Mo) |

## Analyse : ce que les chiffres prouvent

1. **Notre chaîne `.tw` n'est pas un goulet** : parse (1,8 ms pour 600
   blocs), sérialisation (0,2 ms) et lecture de 1 000 notices (1,2 ms)
   sont négligeables face à la compilation Typst.
2. **Le coût dominant est la compilation Typst** (31 ms pour 9 pages), et
   il **grandit avec la bibliographie** : +50 ms pour 1 000 notices —
   c'est le style bibliographique qui traite chaque notice, dans Typst,
   pas dans notre code.
3. **Le coût est déjà contenu par l'architecture**, qui est celle que la
   mesure recommande : compilation unique vers une structure en mémoire,
   rasterisation des seules pages visibles (~3–11 ms/page), double tampon
   d'images par empreinte (`image_hashes`, jamais de purge), et **pas de
   recompilation sans changement** (gardes d'empreinte : « une seule
   compilation à la fois », saut si le texte est inchangé — `src/app/mod.rs`
   autour de 2338–2366). L'application ne recompile pas par frame : elle
   recompile à la pause de frappe, en tâche, sans bloquer l'interface.
4. La ligne du banc frère « compilation = 197 % du budget frame 60 Hz »
   mesure une recompilation **par frame**, que l'application ne fait pas —
   c'est le rappel de pourquoi les gardes existent.

## Décision

**Aucun goulet dans notre code n'est prouvé : rien n'est changé à la
légère.** Les deux leviers connus, si un jour la croissance les rend
nécessaires, sont mesurés et nommés :

- le traitement bibliographique de Typst (50 ms pour 1 000 notices à
  chaque recompilation) — un cache à ce niveau serait du travail sur le
  `World` de Typst, à envisager avec une mesure avant/après ;
- la recompilation complète après chaque frappe (21–31 ms) — un cache
  incrémental Typst est le même genre de chantier.

Pour la bêta, la bibliothèque de l'utilisateur (thèse : quelques centaines
de notices) reste dans les 10–30 ms de compilation : confortable.

## Limites connues

- Mesures sur une machine (Linux x86_64) ; Windows/macOS à refaire sur
  leurs machines natives (R7).
- Le pic mémoire (~96 Mo) est celui du processus de test entier, bancs
  compris — pas le profil isolé de l'application en édition.
