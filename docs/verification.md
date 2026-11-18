# Vérifier Typst Writer

## Les portes ordinaires

À lancer depuis la racine du dépôt :

    cargo fmt --check
    cargo test
    cargo clippy --all-targets -- -D warnings
    cargo build --release
    cargo deny check
    git diff --check

La CI ajoute `cargo test --release`, la batterie applicative et les tests du
crate éditeur vendoré.

## Les épreuves à état

Les tests marqués `#[ignore = "HOME jetable…"]` ne doivent jamais tourner dans
le vrai `HOME` : ils ouvrent une session, écrivent des projets ou touchent aux
versions bibliques. La batterie leur donne un `HOME` neuf par test :

    cargo test --no-run
    python3 scripts/batterie-epreuves.py \
        --prefix app::espace_runtime::epreuves:: \
        --env TYPST_WRITER_TEST_ESPACE=1 \
        --timeout 60 \
        --binary target/debug/deps/typst_writer-<hash>

La ligne `test result: ok` de chaque épreuve fait foi ; l'absence de verdict
est un échec.

## Mesurer le budget de l'aperçu

Le banc `budget_apercu_dix_pages` mesure un document neuf de moins de dix
pages, sans accéder au vrai `HOME` :

    cargo test --release -- --ignored budget_apercu_dix_pages --nocapture

Il imprime la compilation, l'export du PDF depuis le document déjà compilé,
et une bande visible à 1,5 et 3 px/pt. Le budget visé est de 100 ms pour la
compilation et de 200 ms pour une mise à jour au défilement ou au zoom.

## Les épreuves qui dépendent d'un poste

Les autres tests ignorés se classent ainsi :

- **Zotero** : lisent le dossier de données local ou le serveur de Zotero ;
  ils ne tournent que sur un poste où Zotero est installé et configuré.
- **Moteur Typst et PDF** : certains compilent réellement et demandent les
  polices ou `pdftotext` ; la CI installe `poppler-utils` et les polices
  nécessaires.
- **LibreOffice** : l'expérience DOCX exige `soffice` ; elle est lancée à la
  main.
- **Serveur de langage** : les épreuves qui parlent à tinymist ne tournent
  pas dans la CI.

Les tests ordinaires doivent rester hermétiques : pas de vrai `HOME`, pas de
réseau, pas de projet de l'auteur.

## Plateformes

Linux est la plateforme de référence. macOS et Windows sont construits à
chaque poussée ; macOS exécute les tests, Windows saute encore quelques
épreuves liées aux conventions de chemins et à l'extraction du PDF. Ces sauts
sont explicites dans `.github/workflows/ci.yml` ; ils ne valent pas support
complet.
