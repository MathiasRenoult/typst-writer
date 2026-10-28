//! La grammaire de l'éditeur `.tw`.
//!
//! Le markdown de la couche simple, et ses formules mathématiques :
//! `$…$` en ligne, `$$…$$` à part — des bandes opaques, où rien ne se
//! joue pour le reste de la ligne. Les marqueurs du langage sont ceux du
//! markdown ; les formules ajoutent la sienne.
//!
//! Le corps d'une formule est du math Typst : il se colore comme un
//! tout, et ne ferme rien du voisinage.

/// La définition `sublime-syntax`, enregistrée sous l'extension `tw`.
///
/// Le markdown de la couche simple est inclus tel quel — c'est lui qui
/// colore le texte, les titres, les listes. Les formules, elles, sont
/// portées par un **prototype** : ces règles suivent le texte jusque dans
/// les contextes du markdown, et y voient `$…$` et `$$…$$` là où le
/// markdown ne verrait que du texte. Une formule est une bande opaque :
/// ce qui s'y trouve ne ferme rien du voisinage.
pub const GRAMMAIRE_TW: &str = r#"%YAML 1.2
---
# La langue de l'éditeur : le markdown de la couche simple, ses
# marqueurs, et les formules mathématiques.
name: TW
file_extensions: [tw]
scope: text.tw
contexts:
  main:
    - match: '(?=.)'
      push: corps
      with_prototype:
        - match: '^\$\$\s*$'
          scope: punctuation.definition.math.begin.tw
          push: corps-math-a-part
        - match: '\$'
          scope: punctuation.definition.math.begin.tw
          push: corps-math

  corps:
    - include: scope:text.html.markdown

  corps-math:
    - match: '\$'
      scope: punctuation.definition.math.end.tw
      pop: true
    - match: '[^$\n]+'
      scope: markup.math.tw

  corps-math-a-part:
    - match: '^\$\$\s*$'
      scope: punctuation.definition.math.end.tw
      pop: true
    - match: '.+'
      scope: markup.math.tw
"#;

#[cfg(test)]
mod tests {
    use syntect::parsing::SyntaxSet;

    fn ensemble() -> SyntaxSet {
        let mut constructeur = two_face::syntax::extra_newlines().into_builder();

        constructeur.add(
            syntect::parsing::SyntaxDefinition::load_from_str(super::GRAMMAIRE_TW, true, None)
                .expect("la grammaire se charge"),
        );

        constructeur.build()
    }

    /// Les formules se colorent, en ligne et à part — et le reste du
    /// markdown se colore toujours.
    #[test]
    fn les_formules_se_colorent() {
        use syntect::parsing::ScopeStack;

        let ensemble = ensemble();
        let definition = ensemble
            .find_syntax_by_extension("tw")
            .expect("la grammaire tw est là");

        let portees = |ligne: &str| -> String {
            let mut pile = ScopeStack::new();
            let mut vues = String::new();

            for (_, operation) in syntect::parsing::ParseState::new(definition)
                .parse_line(ligne, &ensemble)
                .expect("la ligne se parse")
            {
                pile.apply(&operation);
                vues.push_str(&pile.to_string());
                vues.push(' ');
            }

            vues
        };

        assert!(
            portees("Soit $x^2$ dans le texte.").contains("math"),
            "la formule en ligne est colorée : {}",
            portees("Soit $x^2$ dans le texte.")
        );

        assert!(
            portees("$$").contains("math"),
            "le bloc à part est annoncé : {}",
            portees("$$")
        );

        // Le markdown se colore toujours : un titre reste un titre.
        assert!(
            portees("# Titre").contains("heading"),
            "le markdown survit : {}",
            portees("# Titre")
        );
    }
}
