# Expérimentation DOCX / Markdown — R8

Date : 2026-11-02. Objectif (feuille de route R8, livrable 2) : mesurer la
conversion, les tableaux, les notes, les citations, les images et le
round-trip sur des exemples représentatifs ; livrer un import ou un export
annoncé avec ses limites s'il est suffisamment fiable, sinon documenter
l'alternative. **Le PDF reste la référence de rendu.**

## Procédure reproductible

La sonde est dans le dépôt : `src/langage/generer.rs`, module
`experience_docx`, test `experience_docx_aller_retour`. Elle se rejoue par :

    cargo test experience_docx_aller_retour -- --ignored --nocapture

La fixture représente les cinq axes : deux titres (niveaux 1 et 2), un
paragraphe portant une note `^[la note attachée]` et une citation
`@dupont2023[p. 45]`, un tableau 2×3, une image avec légende. La chaîne
mesurée est `.tw → HTML → DOCX → HTML`, chaque étape dans son propre
dossier (jamais le même nom de fichier en entrée et en sortie — la sonde
se lirait elle-même).

## Matériel

- Linux 7.2.8-9-omarchy x86_64
- LibreOffice 26.8.1.1 (680, Build: 1) — `soffice --headless`
- rustc 1.97.1, dépôt typst-writer 0.3.0-alpha.3

## Mesures (relevé de la sonde)

SOURCE : 2 titres, 6 cellules, 1 note, 1 citation, 1 image.

| Axe        | Aller-retour DOCX (par HTML)                                |
|------------|-------------------------------------------------------------|
| Titres     | OK — texte présent, 2 balises de titre au retour            |
| Tableaux   | OK — les 6 cellules, 1 `<table>`, `<td>/<th>` au retour     |
| Notes      | **Transformées** — le texte survit, en texte nu `[note: …]`, plus aucune identité de note (pas de note de bas de page DOCX) |
| Citations  | OK — la chaîne `@dupont2023[p. 45]` revient intacte, en texte |
| Images     | OK — `<img>` et légende présents                            |

Mécanisme de la perte de note : le HTML n'a pas de vraie note de bas de
page ; la note voyage comme `<span>` dans le paragraphe, et le DOCX
résultant la porte en texte courant. Au retour, rien ne permet de
distinguer une note d'un texte ordinaire.

## Décision

1. **Markdown : export livré** (ou à livrer par cette décision). Le
   dialecte `.tw` **est** du Markdown étendu : titres, tableaux, images et
   listes sont compatibles ; les notes `^[…]` et citations `@clé[…]` se
   gardent telles quelles, lisibles mais non standard. Aucune dépendance,
   fidélité proche de l'identité.
2. **DOCX aller : export livré, limites annoncées.** La chaîne
   `.tw → HTML → DOCX` (LibreOffice) porte titres, tableaux, citations et
   images avec légendes. Limite : les notes deviennent du texte
   `[note: …]`, ce n'est pas une note de bas de page Word. LibreOffice est
   requis ; sans lui, le geste est dit plutôt que manqué en silence.
3. **DOCX retour (import) : non livré.** Les notes perdent leur identité
   (le retour ne les distingue plus d'un texte courant) — un import livré
   mentirait sur ce qu'il rapporte. Alternative documentée : ouvrir le
   DOCX dans LibreOffice Writer, puis exporter en texte ou HTML pour le
   coller dans un projet ; ou pandoc côté utilisateur pour un aller-retour
   plus fidèle, mesuré ailleurs si besoin.

## Limites connues

- Mesures sur une fixture synthétique de cinq axes ; un document réel
  (figures longues, notes multiples, bibliographie) mériterait un second
  relevé avant de promettre quoi que ce soit au-delà de ces chiffres.
- LibreOffice convertit en ré-encodant parfois les blancs (coupures de
  ligne en milieu de phrase) : les mesures de texte sont à comparer sans
  tenir compte des blancs.
- La chaîne DOCX dépend de la version de LibreOffice ; les chiffres
  ci-dessus portent la leur.
