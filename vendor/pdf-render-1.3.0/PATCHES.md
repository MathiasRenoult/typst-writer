# Correctifs de typst-writer sur `pdf-render 1.3.0`

Amont : [pdfluent/pdfluent-sdk, crate `pdf-render`](https://crates.io/crates/pdf-render)
(fork de hayro, Apache-2.0 OU MIT). Copie conforme de la version 1.3.0,
un seul correctif, motivé par bug-report-9 (la fenêtre de rendu ne doit
rasteriser que ce qu'elle montre).

## `origin` — le rendu par région

`RenderSettings` reçoit un champ `pub origin: (f32, f32)` : le point de la
page, en pixels de sortie, à placer en haut à gauche de la toile. La
transform de départ est translatée d'autant, et les champs `width` /
`height` déjà présents bornent la toile — ensemble, ils décrivent une
**région** de la page.

- `(0.0, 0.0)` est neutre : la sortie est **byte-identique** à celle de
  l'amont (`origin` n'entre dans rien d'autre).
- Rendu voulu : la fenêtre visible d'un lecteur PDF à un zoom donné, sans
  rasteriser les 90 % de la page hors écran.

Sans ce correctif, `pdf-render` ne rend que des pages entières (ou leur
coin supérieur gauche), et le coût du rendu croît en densité² — mesuré à
1,9 s par page à 10 px/pt là où la fenêtre visible n'en coûte qu'une
fraction.

Trois entailles au diff : la déclaration du champ, sa valeur neutre dans
le `Default`, et la translation dans `render_with_glyph_cache`.
