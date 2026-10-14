//! Client LSP pour tinymist, écrit pour l'application.
//!
//! Trois raisons de l'écrire ici plutôt que de réutiliser la couche du crate
//! d'éditeur vendoré :
//!
//! 1. **Ce qu'on y reçoit.** Son événement de complétion ne porte que des
//!    étiquettes (`Completion { items: Vec<String> }`) — ni texte d'insertion,
//!    ni plage de remplacement, ni détail, ni documentation. Or les éléments de
//!    tinymist sont **complets** : il n'annonce pas de `resolveProvider`, donc
//!    tout ce qu'il faut pour insérer correctement est déjà dans la réponse. Et
//!    il le faut : insérer une complétion, ce n'est pas ajouter du texte à la
//!    position du curseur, c'est **remplacer** ce qui a été tapé — `#fig`
//!    devient `#figure(…)`, la plage exacte à remplacer ne se devine pas.
//!
//! 2. **Ce qu'on n'y reçoit pas.** Il n'a aucune variante pour
//!    `textDocument/publishDiagnostics` : les erreurs du serveur y sont
//!    ignorées.
//!
//! 3. **Un client, ce n'est pas une couche de plus dans un crate de 35 000
//!    lignes.** Pour un éditeur qui n'ouvre qu'un document à la fois, le
//!    protocole tient en quelques centaines de lignes, qu'on peut éprouver
//!    contre le vrai serveur — ce qui compte, puisque c'est par là que passent
//!    les positions, et que le protocole les compte en unités de code
//!    **UTF-16** quand tout notre code compte en octets UTF-8.
//!
//! Le serveur est **facultatif** : sans lui, l'application reste entièrement
//! utilisable — l'aperçu compile par ses propres moyens, et seule l'assistance
//! disparaît. C'est [`Session::start`] qui le dit, en échouant.

// Le module est complet et éprouvé contre le vrai serveur ; sa mise en service
// dans l'application est l'étape suivante, et cet `allow` disparaît avec elle.
// Il n'existe que pour ne pas couvrir de bruit une compilation où rien de tout
// cela n'est encore appelé.
#[allow(dead_code)]
pub mod client;
#[allow(dead_code)]
pub mod messages;
