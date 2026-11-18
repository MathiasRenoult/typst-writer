//! Les fichiers de showybox 2.0.4, embarqués dans le binaire.
//!
//! Le paquet est vendu avec l'application : un `{callout}` se compose sans
//! réseau et sans dépendre d'un dossier de paquets déjà peuplé. Le tableau
//! est la seule liste des fichiers à poser ; le point d'entrée `showy.typ`
//! fait foi pour reconnaître une installation déjà faite.

/// Les fichiers de showybox, dans l'ordre où ils sont écrits.
pub(crate) const SHOWYBOX_FICHIERS: &[(&str, &str)] = &[
    (
        "showy.typ",
        include_str!("../../vendor/showybox/2.0.4/showy.typ"),
    ),
    (
        "lib/func.typ",
        include_str!("../../vendor/showybox/2.0.4/lib/func.typ"),
    ),
    (
        "lib/id.typ",
        include_str!("../../vendor/showybox/2.0.4/lib/id.typ"),
    ),
    (
        "lib/pre-rendering.typ",
        include_str!("../../vendor/showybox/2.0.4/lib/pre-rendering.typ"),
    ),
    (
        "lib/sections.typ",
        include_str!("../../vendor/showybox/2.0.4/lib/sections.typ"),
    ),
    (
        "lib/shadows.typ",
        include_str!("../../vendor/showybox/2.0.4/lib/shadows.typ"),
    ),
    (
        "typst.toml",
        include_str!("../../vendor/showybox/2.0.4/typst.toml"),
    ),
    (
        "LICENSE",
        include_str!("../../vendor/showybox/2.0.4/LICENSE"),
    ),
];
