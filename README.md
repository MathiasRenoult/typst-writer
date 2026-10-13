# Typst Writer

Éditeur de documents Typst (compilateur + moteur de rendu Typst) destiné à la
rédaction d'assignments de théologie, inspiré de Neo. Interface graphique avec
Iced, édition de code avec iced-code-editor, dialogues avec rfd.

## Statut

- Arborescence, toolbar, sidebar, editor et status bar : implémentés.
- Cycle de base (Nouveau/Ouvrir/Sauvegarder/Fermer) : opérationnel.
- Prévisualisation, compilation Typst, gestion de projet, bibliographie :
  **non commencés** (modules `src/typst`, `src/project`, `src/settings`, `src/lsp`
  sont des stubs vides).

## Développement

```sh
cargo build      # build en debug
cargo run        # lance l'application
typst --version  # vérifier l'outilchaine (0.15.1 installé)
```

## Roadmap

1. Compilation + prévisualisation en temps réel
2. Gestion de projet (dossiers, étagères « shelf »)
3. Bibliographie connectée à Zotero
4. Autocomplétion / LSP (pilote d` tower-lsp)
5. Rafraîchissements UX et persistance

## Licence

Intended for personal use.
