Si tu veux publier ton application sur GitHub en public sans t'attirer d'ennuis juridiques, la bonne nouvelle est que tu peux généralement publier une application écrite en Rust avec iced, utilisant Typst et Tinymist, sans problème particulier. Il faut toutefois distinguer trois choses : les licences des dépendances, ce que tu distribues réellement et les droits sur ton propre code.

Le fait d'utiliser un outil libre ne signifie pas que tout ce qui l'entoure est automatiquement libre. Les licences ont cette petite manie de ne pas se soucier de nos bonnes intentions.
1. Vérifier les licences de tes dépendances
Voici les principaux composants de ton projet :









Rust et ses crates

Le langage Rust et une grande partie de son écosystème sont publiés sous des licences permissives, notamment MIT et Apache-2.0. Vérifie néanmoins les licences de toutes les crates réellement utilisées.












iced

Le projet est sous licence MIT. En règle générale, tu peux l'utiliser dans une application publique, y compris commerciale, à condition de respecter les obligations de sa licence, notamment la conservation de la notice de copyright et du texte de licence.












Typst

Typst possède son propre écosystème de licences et de composants. Vérifie les licences du moteur, des bibliothèques et des ressources que tu embarques. L'utilisation de Typst comme dépendance ne t'oblige pas, à elle seule, à publier ton application sous une licence particulière.












Tinymist

C'est un point à examiner séparément : tu peux utiliser Tinymist comme outil externe installé sur le système, ou incorporer certains de ses composants à ton application. Les conséquences en matière de distribution ne sont pas nécessairement identiques.



Les informations officielles confirment que iced est sous MIT, Typst sous Apache-2.0 et Tinymist sous Apache-2.0. Ces licences sont permissives et permettent généralement la publication de ton application, y compris commerciale, sous réserve de leurs conditions respectives. 


GitHub
+2



2. La checklist avant de publier
Voici ce que je ferais concrètement, dans cet ordre.
1. Vérifier que tu as le droit de publier ton propre code

Assure-toi que le code t'appartient, qu'il n'a pas été copié d'un projet sous une licence incompatible avec ton projet, et qu'aucun contrat professionnel ne te cède les droits sur le travail réalisé.


2. Ajouter une licence à ton dépôt

Si tu souhaites que d'autres puissent utiliser, modifier et redistribuer ton application, choisis une licence explicite, par exemple MIT ou Apache-2.0. Si tu veux seulement rendre le code visible sans autoriser clairement sa réutilisation, ne confonds pas dépôt public et logiciel open source.


3. Auditer toutes les dépendances

Dans un projet Rust, Cargo.toml ne suffit pas : il faut également examiner les dépendances transitives et les fonctionnalités activées.


4. Vérifier les ressources embarquées

Polices, icônes, images, thèmes, exemples de documents Typst et modèles de mise en page peuvent avoir des licences distinctes de celle de ton code.


5. Éliminer les secrets et les données personnelles

Ne publie ni jetons d'accès, ni clés API, ni fichiers de configuration personnels, ni documents privés. Vérifie aussi l'historique Git : supprimer un secret du dernier commit ne l'efface pas des commits précédents.


6. Documenter les dépendances et les crédits

Ajoute un README avec les instructions de compilation, les licences utilisées et les éventuelles mentions de copyright exigées. Si tu distribues le compilateur Typst ou des composants intégrant ses dépendances, vérifie notamment les exigences de conservation du fichier NOTICE. 


GitHub
+1






3. Les commandes à exécuter dans ton projet Rust
Puisque tu développes sous Linux avec Rust, tu peux automatiser une bonne partie des vérifications.

Installe cargo-deny, qui contrôle notamment les licences, les vulnérabilités connues, les crates interdites et les sources de dépendances. 


Rust
+1



cargo install --locked cargo-deny
cargo deny init
cargo deny check


Si une vérification échoue, ne modifie pas aveuglément la configuration pour faire disparaître l'erreur. Examine la dépendance concernée, sa licence et les conditions applicables.

Vérifie aussi les fichiers qui risquent de partir sur GitHub :
git status --short
git ls-files
git log --all --oneline


Le dernier commande permet de repérer l'historique à examiner, mais ne détecte pas à lui seul les secrets. Pour la sécurité, active le secret scanning et la protection contre les dépôts accidentels de secrets lorsque ces options sont disponibles. GitHub documente ces protections pour les dépôts publics. 


GitHub Docs
+1



4. Cas particulier : Tinymist comme outil externe
C'est probablement le point technique le plus important pour ton architecture.
Utilisation	Conséquence générale
Ton application lance un exécutable Tinymist installé séparément	Tu distribues principalement ton propre code et les instructions d'installation.
Tu distribues aussi le binaire Tinymist	Tu dois respecter sa licence et celles des composants redistribués.
Tu intègres le code de Tinymist dans ton application	Il faut analyser les conditions de la licence Apache-2.0 et les éventuels composants tiers intégrés.
Tu utilises directement les crates de Typst	L'utilisation commerciale est permise sous Apache-2.0, avec les obligations applicables à cette licence.  Typst +1

Mon conseil : si Tinymist fonctionne déjà comme un processus externe que ton application Rust communique via LSP, conserve cette séparation. Elle simplifie la maintenance et évite d'embarquer inutilement du code de l'outil dans ton application. Ce n'est toutefois pas une obligation juridique.
5. Quelle licence choisir pour ton application ?
MIT

Simple et permissive. Les autres peuvent réutiliser ton code, y compris commercialement, en respectant les notices requises.


Apache-2.0

Permissive également, avec des dispositions explicites sur les brevets et des obligations de conservation des notices applicables.


GPL-3.0

Si tu veux imposer aux redistributions couvertes par la GPL de respecter ses obligations de partage du code source correspondant. Ce n'est pas une garantie que toute utilisation du programme entraîne automatiquement la publication de modifications.


Pour une application personnelle en Rust, je choisirais MIT ou Apache-2.0, selon que tu privilégies la simplicité ou les dispositions supplémentaires d'Apache. Tu peux aussi garder ton dépôt public sans licence, mais cela ne donne pas aux autres une autorisation générale de réutiliser ton code.
6. Le plan minimal que je suivrais
- [ ] Choisir MIT ou Apache-2.0 pour ton code.
- [ ] Ajouter LICENSE et un README.
- [ ] Exécuter cargo deny check et examiner les résultats.
- [ ] Vérifier les polices, icônes, modèles Typst et autres ressources.
- [ ] Inspecter l'historique Git et exclure les secrets.
- [ ] Activer les protections de sécurité GitHub.
- [ ] Documenter les versions et les méthodes d'installation de Typst et Tinymist.
Les liens officiels les plus utiles :
- Licence et dépôt d’iced
- Licences et utilisation de Typst
- Dépôt et licence de Tinymist
- Documentation de cargo-deny
- Sécurité des dépôts GitHub
Dernière distinction importante : publier le code source sur GitHub et distribuer des exécutables compilés sont deux opérations différentes. La seconde nécessite notamment de vérifier les licences des bibliothèques et ressources réellement embarquées dans tes binaires.

Pour ton projet, le risque le plus probable n'est pas iced ou Typst eux-mêmes. C'est plutôt une dépendance oubliée, une police redistribuée sans autorisation, un secret dans l'historique ou l'absence de licence claire pour ton propre code.