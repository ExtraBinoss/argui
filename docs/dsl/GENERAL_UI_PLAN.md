# Argui DSL — Plan de consolidation et d’UI générale

Date : 22 septembre 2026. Statut : implémentation en cours ; voir les preuves dans la checklist.
Suivi opérationnel : [checklist](GENERAL_UI_CHECKLIST.md).

## Objectif et critère de réussite

Permettre de créer des interfaces originales, réutilisables et accessibles en
`.argui`, avec le même comportement en compilation AOT et en développement live.
Une nouvelle interface doit pouvoir assembler les primitives existantes ; une
surface spécialisée doit pouvoir être ajoutée par une crate applicative sans
modifier le framework.

La livraison doit démontrer ces capacités sur trois applications de preuve :

1. Une table éditable : données réactives, tri/filtre, cellules personnalisées,
   sélection, validation, dialogue à plusieurs slots et chargement asynchrone.
2. Un éditeur de nœuds : déplacement, connexions vectorielles dynamiques,
   zoom, sélection, raccourcis et intégration d’une surface native externe.
3. Un formulaire multilingue : français/arabe, pluriels, texte riche, disposition
   responsive, erreurs accessibles et parcours complet au clavier.

Le nombre de pages de galerie n’est pas un critère de complétude. Chaque
capacité doit avoir un contrat, un exemple réel et des preuves de comportement.

## Point de départ de l’audit

Les composants composés en DSL, les propriétés réactives, les thèmes, les
primitives d’interaction, les portails, les assets, les shaders, la virtualisation
et les animations sont déjà présents. Préserver ces acquis et leurs tests.
Les anciennes checklists contiennent des états devenus ambigus : vérifier le
code avant de considérer une capacité absente ou terminée.

L’audit initial est une inspection du code et des tests, complétée par des sondes
avec le binaire CLI déjà présent. Ce n’est pas une nouvelle validation Nextest,
graphique ou multiplateforme. Les sondes ont montré que `argui check` accepte une
affectation string vers int dans un handler, une entrée sur une propriété privée
d’enfant et un repeater sans clé. Le chemin vectoriel dynamique est explicitement
rejeté. Reproduire ces cas avec les sources courantes avant correction.

Principaux points d’entrée :

| Domaine | Sources à examiner |
| --- | --- |
| Validation et expressions | `crates/argui-dsl/dsl-semantic/src/check/`, `dsl-parser/src/grammar/` |
| Lowering et contrats | `crates/argui-dsl/dsl-ir/src/visual.rs`, `model.rs` |
| Backend AOT | `crates/argui-dsl/dsl-compiler/src/codegen/` |
| Backend live et migration | `crates/argui-dsl/dsl-runtime/src/` |
| Primitives | `crates/argui-schema/src/builtin/`, `builtin.rs` |
| Capacités moteur | `crates/argui-ui/`, `argui-layout/`, `argui-text/`, `argui-accessibility/` |
| Bibliothèque et preuves | `crates/argui-dsl/stdlib/ui/`, `app_examples/widget-gallery-dsl/` |

## Règles d’architecture

- Rust conserve plateforme, rendu, layout, routage d’événements, focus, édition,
  modèles, services et tâches. Le moteur reste indépendant des crates DSL.
- La DSL possède composition, présentation, états et comportement des contrôles.
- Aucun dispatch par nom de widget standard dans le compilateur ou le runtime.
- Une capacité traverse tout le chemin applicable : syntaxe, schéma, validation,
  IR, AOT, live, reload, tooling, documentation et tests.
- Les effets GPU ne remplacent pas les contrats de layout, hit testing ou
  accessibilité. Les services réseau et disque restent côté Rust.
- Réutiliser les mécanismes moteur existants. Ajouter une abstraction seulement
  lorsqu’un besoin concret des applications de preuve la justifie.
- Retirer les API remplacées avec leurs appelants, sans couche de compatibilité
  permanente. Migrer les exemples existants au cours du chantier concerné.
- Les changements de code natif ou d’ABI publique nécessitent un rebuild explicite ;
  les changements DSL compatibles restent rechargeables sans redémarrage.

## Ordre d’exécution

Ordre recommandé : **P0 → P1 → P2 → P3 → P4 → P5 → P6 → P7 → P8 → P9 → P10**.
Les identifiants désignent des phases, pas des niveaux de gravité.

| Phase | Dépendances | Résultat attendu |
| --- | --- | --- |
| P0 | Aucune | Baseline et reproductions fiables |
| P1 | P0 | Sémantique et exécution cohérentes |
| P2 | P1 | Composition et styles complets |
| P3 | P1 | Layout et géométrie visuelle exposés |
| P4 | P1, P2 | Expressions et modèles exploitables |
| P5 | P1, P4 | API Rust commune et extensions natives |
| P6 | P3, P4, P5 | Dessin réactif et gestes composables |
| P7 | P2, P3, P5 | Accessibilité, texte et internationalisation |
| P8 | P1 à P7 | Reload structurel et outils fiables |
| P9 | P2 à P8 | Trois applications de preuve complètes |
| P10 | P9 | Validation finale et documentation de livraison |

Ne pas attendre P9 pour vérifier visuellement les changements : les scénarios
ciblés accompagnent chaque phase. Corriger une divergence AOT/live dans la phase
qui l’introduit, avant de construire les phases suivantes dessus.

## P0 — Établir la baseline

Lire les règles de contribution, relever le commit et l’état de travail,
inventorier les tests existants et distinguer les mécanismes déjà implémentés des
preuves encore manquantes. Construire des fixtures minimales pour les anomalies
de l’audit ; les tests doivent viser le comportement public.

Reproduire aussi le court-circuit booléen, les slots supplémentaires non remplis,
l’application des styles et le retrait d’un enfant monté pendant un reload.
Le dernier cas est une déduction statique à confirmer, pas un résultat exécuté.

**Sortie :** baseline datée, tests de reproduction identifiés, capacités et
limitations observées consignées dans la checklist.

## P1 — Rendre le langage fiable

Valider les statements complets : destination assignable, direction des propriétés,
compatibilité des types, opérateurs composés, retours et bindings bidirectionnels.
Interdire l’affectation externe des propriétés privées ou de sortie.

Rendre `key` obligatoire pour les repeaters : une omission doit produire une
erreur localisée, jamais une disparition silencieuse. Définir et vérifier le
traitement des clés dupliquées et leur portée par instance.

Implémenter un court-circuit identique pour `&&`, `||` et les conditionnelles.
Réserver les callbacks à effets aux handlers ; distinguer les fonctions pures
utilisables dans les bindings. Spécifier conversions numériques, unités, valeurs
optionnelles et erreurs arithmétiques, puis vérifier la parité des backends.

Préserver les warnings dans les sorties CLI et LSP. Un programme rejeté doit
produire un diagnostic DSL utile, pas une erreur obscure du Rust généré.

**Sortie :** corpus commun de conformité exécutant des programmes valides et
invalides ; les exemples acceptés compilent réellement en AOT et s’exécutent avec
les mêmes résultats en live. Un simple test de chaîne Rust générée ne suffit pas.

## P2 — Composition, références et styles

Définir une syntaxe explicite de fourniture des slots nommés, leur cardinalité,
leur contenu par défaut et leur forwarding. Permettre des templates à paramètres
typés pour les cellules et lignes, sans évaluer d’avance une liste virtualisée.

Définir les références lexicales aux enfants dans les repeaters et les références
optionnelles aux enfants conditionnels. Préserver leur isolation par instance et
interdire les lectures qui ne peuvent pas être résolues de façon sûre.

Ajouter une application explicite des styles nommés jusqu’au rendu AOT/live.
Spécifier et tester la précédence entre défauts, thèmes, styles, valeurs inline,
états et présentation animée. Documenter la portée des surcharges de tokens et
les limites intentionnelles, sans introduire une cascade implicite.

**Sortie :** `Dialog` avec header/body/actions, champ avec leading/trailing/error,
et ligne personnalisée avec référence locale ; styles partagés réellement
observables et modifiables en live, sans branche spéciale pour ces composants.

## P3 — Layout et propriétés visuelles générales

Exposer depuis le moteur Grid, tracks, fractions, min/max, spans, placement,
tailles maximales, aspect ratio, flex basis, alignements et espacements par côté.
Exposer les règles responsive basées sur la taille du conteneur et définir le
contrat de lecture des dimensions mesurées sans boucle de layout.

Rendre cohérents les types de dimensions et unités. Définir le positionnement,
la superposition, le clipping et les transformations composées, avec origine,
échelle et translation. Exposer les rayons indépendants, bordures par côté et
paramètres d’ombre nécessaires aux preuves.

**Sortie :** dashboard à grille et formulaire responsive sans calculs de positions
spécifiques en Rust ; contrôle des tailles, du débordement, de la peinture et des
zones interactives aux différentes tailles et échelles d’affichage.

## P4 — Expressions et données réactives

Ajouter un ensemble cohérent et limité : indexation sûre, littéraux de structs
et d’enums, manipulation explicite des optionnels, variables locales réelles,
branches dans les handlers et fonctions pures typées. Documenter les erreurs et
les règles de dépendance ; ne pas transformer la DSL en langage système.

Distinguer `array<T>` de `model<T>`. Définir une interface modèle commune aux deux
backends : identité stable, lecture de ligne, insert/remove/move/update et
notifications. Définir ownership, abonnement, destruction et migration.

Fournir tri/filtre sous forme de projections adaptées aux preuves, en réutilisant
les capacités Rust lorsque pertinent. Maintenir la virtualisation existante et
ajouter la mesure des lignes de hauteur variable pour le cas de contenu riche.
Préserver sélection, focus et position d’ancrage lors des mutations.

**Sortie :** table avec données modifiables et liste à hauteurs variables ; mesurer
le nombre de lignes montées, les évaluations et les copies lors d’une modification
isolée. Une mise à jour de ligne ne doit pas imposer une reconstruction complète
du modèle à la frontière DSL.

## P5 — Contrat Rust commun, services et extensions natives

Générer une façade publique typée identique pour AOT et live : propriétés,
callbacks, structs, enums, modèles et slots. Garder les identifiants internes et
`DslValue` dans l’implémentation. Supprimer les bridges applicatifs qui recherchent
un callback par position ou inspectent manuellement l’IR.

Relier cette façade aux tâches du runtime : loading/result/error, annulation,
dernière requête gagnante et destruction. Le file picker et la recherche doivent
rester réactifs pendant l’attente ; les résultats expirés ne doivent pas revenir
dans un composant démonté.

Permettre l’injection du même registre de primitives dans compilation, génération,
runtime live et tooling. Définir identité, version de contrat, propriétés,
événements, slots, assets et erreurs de compatibilité. Une nouvelle implémentation
Rust exige un rebuild ; modifier son usage DSL reste compatible avec le reload.

**Sortie :** le même code métier fonctionne en AOT/live ; une crate applicative
enregistre une surface spécialisée reconnue par le LSP et les deux backends sans
modifier les crates Argui.

## P6 — Dessin dynamique et gestes

Ajouter des chemins dont les commandes dépendent de propriétés ou de modèles.
Réutiliser la chaîne vectorielle ; définir validation de géométrie, identité,
cache, invalidation et libération. Conserver le chemin efficace des assets
statiques. Prévoir fill/stroke et édition des points nécessaires aux preuves.

Exposer pan/pinch/rotation avec phases, coordonnées, centre et annulation.
Spécifier capture, propagation et arbitrage avec les zones scrollables. Vérifier
que transformations visuelles et hit testing utilisent des conventions communes.

**Sortie :** une connexion Bézier suit ses deux nœuds sans recompilation ni fichier
asset intermédiaire ; zoom et drag restent corrects dans un conteneur scrollable,
avec invalidation et ressources bornées.

## P7 — Accessibilité, texte et internationalisation

Exposer les valeurs, bornes et actions accessibles, les états pertinents, les
relations label/description/controls/active-descendant, les annonces et les
métadonnées des listes/grilles. Réutiliser les mécanismes moteur existants.

Compléter le clavier des composants standards : Slider avec flèches/Home/End,
menus, tabs, sélection, combobox, dialogs et restauration du focus. Vérifier les
actions accessibles par des tests sémantiques, au-delà de l’existence d’un rôle.

Exposer familles et assets de polices ainsi que des spans typés dans un même flux
de texte. Vérifier wrapping, sélection, IME et positions de caret dans les cas
multilingues. Exposer les arguments de `tr`, le changement réactif de locale et
la direction héritée avec propriétés logiques.

**Sortie :** formulaire français/arabe avec pluriels, styles de texte mixtes,
erreurs reliées aux champs, lecture sémantique correcte et parcours clavier.

## P8 — Reload structurel et outils d’édition

Migrer uniquement les instances encore pertinentes et compatibles. Démonter
proprement les composants supprimés. Tester insertion, suppression, remplacement,
réordonnancement et reparenting avec le contrat explicite des identités `#id` et
des clés. Documenter quels changements conservent ou réinitialisent l’état.

Préserver lorsque valide focus, sélection, scroll, overlays et animations ;
conserver la dernière génération utilisable lorsqu’un nouveau package échoue.
Vérifier le hash d’ABI pour les types publics transitifs et les contrats natifs.

Synchroniser le LSP avec les changements disque et les dépendants des modules
édités. Relier l’inspection d’un élément à son composant, sa source et ses
bindings. Expliquer les erreurs de reload et les causes d’invalidation avec les
outils existants, sans dupliquer un système de devtools.

**Sortie :** modifications structurelles pendant une interaction sans état
transféré à un mauvais élément ; erreurs localisées, reprise après correction et
mise à jour visible sans clic supplémentaire.

## P9 — Applications de preuve et galerie

Construire les trois preuves définies en tête de document avec les contrats des
phases précédentes. Les intégrer aux exemples et réutiliser leurs composants.
Chaque contournement nécessaire doit être identifié et résolu dans le mécanisme
général concerné avant de déclarer la preuve terminée.

Tenir une matrice par capacité/page : implémentée, démonstration partielle ou
service hôte requis ; tests AOT/live, clavier, pointeur, accessibilité, reload et
plateformes effectivement vérifiées. Relier l’inventaire des pages aux tests pour
détecter une page ajoutée sans couverture déclarée.

Comparer aussi layout, focus, sélection et offsets, pas uniquement texte/couleur.
Inspecter des captures non blanches sur l’affichage Linux privé. Vérifier idle,
virtualisation, coût des mutations et ressources après des reloads répétés.

**Sortie :** trois scénarios utilisables de bout en bout et une galerie dont les
annonces correspondent aux comportements démontrés.

## P10 — Livraison et vérification finale

Mettre à jour language, getting-started, development, inventaires et anciennes
checklists pour supprimer les affirmations devenues fausses. Documenter les
contrats publics et les limitations intentionnelles, avec exemples exécutables.

Pendant l’implémentation, exécuter uniquement les crates/tests directement
affectés avec `cargo nextest run -p PACKAGE --all-features` et garder ces features
identiques entre runs ciblés. Les exemples ont leur workspace : utiliser
`--manifest-path app_examples/Cargo.toml` pour leurs tests.

Respecter les [règles de qualité](../contributing/code-quality.md) : fichiers Rust
de 600 lignes maximum, tests sous `tests/`, Rustdoc complète, dépendances dans la
crate consommatrice et aucune couverture LLVM concurrente.

Vérifier le build release, les assets embarqués et l’absence des dépendances de
développement dans le graphe AOT. Vérifier les clients web et mobile réellement
disponibles ; consigner séparément compilation et exécution sur appareil. Une
plateforme non exécutée ne doit pas être présentée comme validée.

Après implémentation terminée et immédiatement avant commit, exécuter le gate
complet exactement une fois, conformément aux règles du dépôt :

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
```

Chaque crate et le total doivent atteindre 85 % sur lignes, fonctions, régions
LLVM et branches. Ne pas relancer le gate complet à chaque phase. Toute erreur
ou vérification bloquée reste visible dans la checklist ; ne jamais cocher un
résultat non obtenu. La création du présent plan ne déclenche ni ce gate ni un
commit.

**Sortie :** preuves référencées, risques restants explicites, diff relu et
checklist correspondant exactement à ce qui a été implémenté et vérifié.
