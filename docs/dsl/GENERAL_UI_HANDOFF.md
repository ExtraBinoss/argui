# Passation — checkpoint DSL au 23 septembre 2026

Branche : `codex/dsl-gallery-live`, départ `d889f4a`. La fusion de
`codex/gallery-text-fidelity` (`2968031`) a été résolue dans ce checkout.
L'utilisateur a demandé d'arrêter le développement DSL et de committer l'état
courant, car un passage à Solid est envisagé. Ce commit est donc un checkpoint,
pas une déclaration d'achèvement de P5 ni une invitation à lancer P6–P10.
Le plan de référence est [GENERAL_UI_PLAN.md](GENERAL_UI_PLAN.md) et les critères
sont dans [GENERAL_UI_CHECKLIST.md](GENERAL_UI_CHECKLIST.md).

## État technique

| Phase | Travail présent | Reste si le DSL est conservé |
| --- | --- | --- |
| P0–P1 | Baseline existante et correctifs principaux ; 18 scénarios AOT/live passaient avant ce checkpoint | Revenir seulement sur un défaut concret. |
| P2 | Slots nommés, défauts, cardinalité, forwarding, templates typés paresseux, références lexicales et conditionnelles ; styles AOT/live et tests de précédence | Documenter formellement la portée des tokens et les limites de la cascade ; revue de régression complète. |
| P3 | Grid, dimensions et espacements, positionnement, transformations, bordures/ombres, container queries ; dimensions mesurées et garde contre les bindings de géométrie circulaires | Captures GUI sur affichage privé, peinture/hit testing sous clipping/transformation, plusieurs échelles. |
| P4 | Indexation sûre, optionnels, littéraux struct/enum, handlers à variables/branches, fonctions pures typées avec expansion IR, `Model<T>` à identités et mutations, liste à hauteurs variables | Préservation prouvée du focus, de la sélection et de l'ancrage scroll ; coûts montages/évaluations/copies ; revue des fonctions pures au-delà des tests ciblés. |
| P5 | Façade live typée, callbacks/propriétés/modèles/slots, suppression du bridge de callbacks par position ; registre d'extension partagé et primitive applicative `ProofTile` ; hôte de recherche/file picker asynchrone ajouté | Test d'intégration fiable du service et validation GUI, ABI d'extension sur changement de version/propriété, frontière rebuild/reload documentée et preuve complète de parité AOT/live. |
| P6–P10 | Aucun développement dans cette conversation | À confier seulement si le chantier DSL continue. |

## Vérifications connues de ce checkpoint

- Après la fusion text-fidelity : 709 tests passés, 2 ignorés, sur les parties fusionnées.
- P2 : 124 tests parseur/sémantique, 4 cardinalité, 10 références, 12 compilation/runtime, 4 exemples slots et 2 de précédence des styles passés en sélections ciblées.
- P3 : preuves Grid et formulaire responsive AOT/live passées ; lecture des dimensions mesurées AOT/live passée ; garde directe testée. Le test responsive de la galerie a été ajusté à la géométrie actuelle et passe isolément.
- P4 : modèles AOT/live et liste riche à hauteurs variables passés en sélections ciblées ; fonctions pures AOT/live 2/2 et diagnostics ciblés 3/3.
- P5 : extension native AOT/live/LSP 4/4, façade live 3/3 et slot hôte AOT/live 2/2. Le service de galerie compile, mais son test d'intégration a révélé une limite du harnais `TestApp` avec un hôte imbriqué et a été retiré ; le comportement du service n'est pas déclaré validé.
- Les chiffres CPU manuels de l'utilisateur avant fusion restent la référence : Button ~0,8 %, Animation Lab 1,5 % maximum et 0 % sans animation, Damage Control ~1,1 %, écart live +0,1–0,2 point. Aucune nouvelle enquête performance n'a été lancée.

Le gate `./scripts/quality.sh` doit être exécuté exactement une fois juste avant
ce commit conformément à `AGENTS.md`. Son résultat et le hash final du commit
sont à lire dans la réponse de passation, car ils ne sont pas connus lors de
l'écriture de ce document.

## Si une nouvelle conversation reprend le DSL

Commencer par décider si le passage à Solid remplace ce chantier. Si le DSL est
conservé, partir de ce checkpoint, vérifier d'abord le service asynchrone de la
galerie et les manques P2–P5 ci-dessus. Ne pas refaire P0 ni l'enquête CPU sans
régression concrète. P6–P10 restent hors du périmètre de ce checkpoint.
