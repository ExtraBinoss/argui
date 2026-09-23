# Baseline du chantier UI générale

22 septembre 2026, worktree `4a82`, commit initial
`d22fb042cca2bdb528275b22df040c487daa0617`, état Git initial propre.
Plan et checklist copiés du worktree `1b43`, où ils étaient préparés.
Règles de qualité et tests Linux privés lues avant modification.

## Reproductions exécutées

- `dsl-semantic/tests/check/statement.rs` : l’écriture d’une entrée privée
  d’enfant et un binding bidirectionnel calculé étaient acceptés ; les clés
  absentes produisaient seulement un warning. Tests rouges avant correction.
- `dsl-runtime/tests/bytecode/short_circuit.rs` : une branche booléenne
  inatteignable appelait son callback ou échouait sur une propriété absente.
  Deux tests rouges avant correction, désormais verts.
- `dsl-runtime/tests/runtime/reload.rs` : supprimer la définition d’un enfant
  privé déjà monté rejetait le reload avec `MissingComponent`. Test rouge avant
  correction, désormais vert avec état racine conservé.

Les premières versions des fixtures de handler utilisaient un nom de primitive
puis un nom d’événement invalides ; elles ont été corrigées en `TouchArea` et
`moved`. Leurs échecs initiaux ne prouvent pas à eux seuls la reproduction des
affectations invalides. Le code antérieur ne vérifiait que les expressions.

## Vérifications

Commande ciblée conservant les features :

```sh
cargo nextest run -p argui-dsl-semantic -p argui-dsl-runtime -p argui-dsl-compiler --all-features --no-fail-fast
```

Première passe élargie : 292 tests, 286 passent ; trois tests attendaient encore
une erreur de codegen au lieu du diagnostic sémantique désormais précoce, deux
utilisaient un retour non-void sur un événement natif, un test de connexion a
dépassé son délai de 200 ms. Ces résultats ne constituent pas une validation
finale. Les fixtures des contrats modifiés sont migrées au fil du chantier.

```sh
cargo nextest run --manifest-path app_examples/Cargo.toml -p argui-example-dsl-live-demo --all-features --test conformance --no-fail-fast
```

Dix-huit tests passent dans la dernière passe ciblée : le Rust AOT est compilé par rustc et le même programme DSL
est exécuté en live. Vérifications : booléens/conditionnelle sans callback
parasite, conversions int/float/optional, affectations composées, concaténation
de chaînes, précision f32, comparaisons i64, texte obtenu après événement, clés
imbriquées après réordonnancement, styles et survol, slots nommés/defaults/forwarding.

Le cache Cargo partagé nécessite l’accès hors sandbox ; cet accès a été accordé
par la revue automatique. Aucun gate global ni couverture LLVM n’a été exécuté à ce stade.
Les 74 tests CLI passaient lors de leur passe ciblée ; une passe ciblée
sémantique/IR/compiler/runtime ultérieure a obtenu 203 réussites sur 204.
L’assertion restante comparait la forme textuelle du Rust généré : elle a été
adaptée, puis le test et un nouveau test de binding remplacé sont passés (2/2).
La régression complète de ces crates reste à rejouer après les modifications.

Une capture Linux privée de la démo live a été inspectée :
`/tmp/argui-general-ui-capture/initial.png`. La fenêtre et les contrôles sont
visibles, mais les Text natifs non colorés sont blancs sur fond blanc.
La démo applique maintenant un style explicite basé sur le token foreground ;
une nouvelle capture est requise. La galerie DSL a aussi été capturée sur
l’affichage privé pendant la navigation et l’animation ; les chiffres CPU sont
consignés dans la checklist. Aucune plateforme autre que Linux n’est annoncée
comme exécutée.
