# Audit des crates pour l'API TSX v2

État du 26 septembre 2026 sur ce checkout. Cet audit étudie le graphe Rust, les exports, les features et les consommateurs visibles. Il ne connaît pas tous les utilisateurs des versions publiées. La rupture assumée du contrat TSX v2 ne prouve pas qu'un export Rust public puisse être retiré sans migration.

## Méthode et limites

- Les 21 membres viennent du [workspace](../../Cargo.toml). Les dépendances et features viennent des Cargo.toml de chaque crate. Les résultats clés ont été vérifiés avec cargo tree --offline -e normal.
- Les volumes ci-dessous sont des lignes physiques Rust sous src/, arrondies à la dizaine. Ils excluent tests/, exemples et assets JS. Ce sont des indicateurs de surface à lire, pas des mesures de temps ou de taille du binaire.
- « Aucun appelant interne » signifie aucun autre manifeste de crate du workspace. Les applications et hôtes générés sont vérifiés séparément dans [l'hôte galerie](../../apps/gallery/quickjs-host/Cargo.toml) et le [modèle natif](../../crates/argui-cli/assets/hosts/native/Cargo.toml). Un consommateur externe peut toujours utiliser la crate.
- Pas de profil CPU, allocations, temps de build froid/chaud, taille release ni inventaire crates.io à ce stade. Les gains annoncés plus bas sont des hypothèses assorties de tests de décision.

Les consolidations déjà présentes sont cohérentes : argui-reactive a disparu du workspace, les shaders sont dans [render/shader](../../crates/argui-render/src/shader/), les entrées Android/iOS dans [runtime/mobile](../../crates/argui-runtime/src/mobile/) et inspect est optionnel. Ces quatre retraits expliquent le passage de 25 à 21 membres.

## Graphe et features : résultats reproductibles

Le chemin moteur est core → paint/text/animation/accessibility → ui → layout/render/platform → runtime. La branche TSX est schema → host → runtime. Theme est obligatoire dans runtime ; inspect et webview y sont optionnels. Le CLI dépend d'automation, qui réunit host, runtime, render, layout et inspect.

1. cargo tree -p argui-runtime --no-default-features --offline -e normal contient theme, host, schema et render, mais **pas** inspect ni webview. Ajouter --features inspect fait apparaître inspect. Une app Rust pure traverse aujourd'hui la branche de dépendances TSX ; le gain à les séparer reste à mesurer.
2. cargo tree -p argui-schema --no-default-features --offline -e normal contient effects → render → wgpu. L'arête vient de [schema/Cargo.toml](../../crates/argui-schema/Cargo.toml), qui demande effects/scroll, et de [builtin/virtual_window.rs](../../crates/argui-schema/src/builtin/virtual_window.rs). Un outil de contrat seul compile donc le graphe GPU. Runtime dépend déjà de render : ce surcoût ne se cumule pas pour l'application complète.
3. cargo tree --manifest-path apps/gallery/quickjs-host/Cargo.toml --no-default-features --offline -e normal contient theme mais ni inspect ni automation. --features automation ajoute ces deux crates. Le Cargo.lock énumère aussi des dépendances optionnelles ; sa présence ne suffit pas à prouver leur inclusion dans une sortie release.
4. [automation/Cargo.toml](../../crates/argui-automation/Cargo.toml) demande core/metrics et layout/metrics ; desktop-metrics ajoute sysinfo séparément. [media/Cargo.toml](../../crates/argui-media/Cargo.toml) ne charge ses décodeurs image/SVG qu'avec media. Les intégrations [platform](../../crates/argui-platform/Cargo.toml) et runtime/{webview,tasks,inspect} sont activées explicitement. Un point d'entrée Rust pratique doit préserver ces choix.

## Inventaire des 21 crates

Les noms de dépendances et d'appelants dans le tableau omettent le préfixe argui-. « Appelants » désigne les autres crates du workspace. Les liens ouvrent les manifestes ; les exports cités sont dans src/lib.rs de la crate.

| Crate (lignes src/ ≈) | Dépend de ; appelants internes | Export et usage constaté | Décision |
| --- | --- | --- | --- |
| [core](../../crates/argui-core/Cargo.toml) (1 550) | aucune ; 15 crates | Géométrie, couleur, entrée, Name et métriques ; base de tous les étages. | Garder la base sans renderer ; metrics optionnel. |
| [paint](../../crates/argui-paint/Cargo.toml) (2 000) | core ; 9 crates, platform optionnel | DisplayList, styles, primitives ; ui/layout/render le consomment. | Garder les données de peinture séparées du GPU. |
| [text](../../crates/argui-text/Cargo.toml) (2 940) | core ; 6 crates | TextEngine, placement, sélection, input ; ui/layout/render/schema l'appellent. | Garder ; refonte seulement avec cas mesuré. |
| [animation](../../crates/argui-animation/Cargo.toml) (3 240) | core ; automation/runtime/schema/ui | Motion, Tween, ressorts, scheduler, timeline. Nombreux types publics. | Garder ; ne pas supprimer un export faute d'usage local. |
| [accessibility](../../crates/argui-accessibility/Cargo.toml) (1 580) | core ; runtime/ui | Arbre sémantique, relations, ponts AccessKit/DOM. | Garder la frontière natif/Web. |
| [ui](../../crates/argui-ui/Cargo.toml) (20 540) | accessibility/animation/core/paint/text ; 7 crates, effects optionnel | Element, UiTree, événements, focus, styles et [réexports](../../crates/argui-ui/src/lib.rs). | Garder la crate, poursuivre les modules par responsabilité ; sa taille ne prouve pas un besoin de fusion/réécriture. |
| [layout](../../crates/argui-layout/Cargo.toml) (6 590) | core/paint/text/ui ; automation/runtime/webview | LayoutEngine, LayoutOutput, régions et profils ; metrics optionnel. | Garder le moteur séparé ; mesurer les scènes v2. |
| [render](../../crates/argui-render/Cargo.toml) (12 030) | core/paint/text ; automation/effects/runtime | RendererDevice, damage, registry, shader ; [lib.rs](../../crates/argui-render/src/lib.rs) réexporte wgpu. | Garder ; le réexport wgpu est un engagement public à auditer avant changement. |
| [platform](../../crates/argui-platform/Cargo.toml) (6 140) | core/paint? ; runtime | Fenêtre, clipboard, préférences et intégrations sous features ; galerie active file-picker. | Garder ; mesurer les features séparément. |
| [runtime](../../crates/argui-runtime/Cargo.toml) (18 920) | 12 obligatoires, inspect?/webview? ; automation | Application, modèles, thème, launch, pont TSX et entrées mobiles ; [exports](../../crates/argui-runtime/src/lib.rs). | Garder comme entrée ; isoler le coût TSX pour Rust pur si mesure favorable (A). |
| [theme](../../crates/argui-theme/Cargo.toml) (1 330) | core/paint ; runtime | ThemeRuntime, tokens, snapshots ; utilisé dans [theme_bridge.rs](../../crates/argui-runtime/src/theme_bridge.rs) et l'environnement Rust. | Garder obligatoire ; profiler snapshots avant optimisation. |
| [schema](../../crates/argui-schema/Cargo.toml) (8 990) | animation/core/effects/media/paint/text/ui ; host/runtime | SchemaRegistry, valeurs, adaptateur et export du contrat. | Garder pour les primitives custom ; découpler export seul du GPU (B). |
| [host](../../crates/argui-host/Cargo.toml) (780) | schema/ui ; automation/runtime | Host, Operation, identité et transaction. Runtime/native_host l'emploie. | Garder ; mesurer matérialisation et clones (C). |
| [inspect](../../crates/argui-inspect/Cargo.toml) (1 200) | core ; automation/runtime? | Snapshots, traces et diagnostics ; runtime seulement avec inspect. | Garder hors graphe release ordinaire. |
| [automation](../../crates/argui-automation/Cargo.toml) (1 350) | animation/core/host/inspect/layout/render/runtime/text/ui ; CLI | Driver, actions et métriques ; argui test et hôtes l'activent. | Garder pour tests, absent des apps release ; coût CLI à examiner (D). |
| [effects](../../crates/argui-effects/Cargo.toml) (730) | paint/render/ui? ; schema | Presets/registre ; schema utilise EdgeFade/EdgeShadow, [galerie](../../apps/gallery/quickjs-host/src/effects.rs) appelle registry. | Garder la capacité ; réduire son couplage au schéma (B). |
| [media](../../crates/argui-media/Cargo.toml) (730) | core/paint ; schema | Handles/registry même sans décodeurs ; [assets-host](../../apps/gallery/assets-host/src/lib.rs) active image/SVG. | Garder ; types d'assets et décodeurs ne sont pas le même coût. |
| [i18n](../../crates/argui-i18n/Cargo.toml) (450) | aucune ; aucun | Catalog/Localizer/Fluent utilisés par [quickjs-host](../../apps/gallery/quickjs-host/src/i18n.rs) et le modèle natif. | Garder malgré zéro arête entre crates. |
| [webview](../../crates/argui-webview/Cargo.toml) (1 520) | core/layout/ui ; runtime? | WebViewPool, backends et politiques utilisés dans [runtime/app/native_views.rs](../../crates/argui-runtime/src/app/native_views.rs) sous webview. | Garder opt-in ; valider plateformes et poids avant défaut. |
| [updater](../../crates/argui-updater/Cargo.toml) (840) | aucune ; aucun | Updater, états, install et HTTP signé sous native ; [exemple](../../crates/argui-updater/examples/startup.rs), aucun hôte/app de production local. | Conserver l'API publiée en attendant enquête d'adoption et de support (E). |
| [cli](../../crates/argui-cli/Cargo.toml) (3 470) | automation sur non-WASM ; aucun | run/run_in, init/build/test/add et binaire [main.rs](../../crates/argui-cli/src/main.rs). | Garder ; mesurer le coût de la dépendance automation (D). |

Les chiffres sont des instantanés ; les autres agents éditent encore le dépôt. La séparation réelle des features est plus importante que le nombre de crates : un appelant de schema n'a pas besoin de payer wgpu, et une application Rust pure devrait pouvoir éviter le pont TSX si ce découplage vaut son coût.

## Code mort : conclusion prudente

**Aucune suppression de source de production n'est prouvée ici.** Une recherche des attributs allow(dead_code) sous crates/**/*.rs n'en trouve que dans deux fixtures de tests du renderer. Ce signal faible n'exclut pas d'autres fonctions inutilisées. Le compilateur et une recherche textuelle ne voient ni tous les cfg/features, ni les versions publiées et leurs consommateurs externes.

| Candidat à instruire, pas à supprimer | Ce que l'on sait | Preuve encore nécessaire |
| --- | --- | --- |
| updater comme produit séparé | Aucun autre manifeste du workspace ni app/hôte local de production ne l'importe ; ses tests et son exemple l'exercent. | Chercher les consommateurs externes, promesse de support et politique de release. Le CLI update vérifie SHA-256 tandis que updater/native est signé : ne pas les confondre. |
| Certains réexports spécialisés d'animation, paint, render, ui | Pas de consommateur local évident pour chaque symbole, mais ce sont des API publiques. | Indexer les symboles par cible/feature et rechercher usages externes connus ; fournir équivalent, migration et versionnement avant retrait. |
| Features desktop peu exercées (tray, gtk-host, native-popups, webview) | Implémentations ciblées présentes, couverture de pages locale inégale. | Compiler/tester chaque plateforme supportée et inventorier les apps clientes. Rareté d'activation ≠ code mort. |
| Clones et conversions dans host/wire/theme | [Stage::materialize](../../crates/argui-host/src/transaction.rs), [WireValue](../../crates/argui-runtime/src/native_host/wire.rs) et [ThemeRuntime::snapshot](../../crates/argui-theme/src/runtime.rs) copient des données. | Profil p50/p95 et allocations sur 100/1 000/10 000 nœuds, mutations de thème et listes virtuelles. Pas de réécriture sans goulot observé. |

## Chantiers proposés, coût et critères de décision

Les durées sont des **ordres de grandeur d'ingénierie**, non des gains mesurés. Elles supposent migration des appels et tests ciblés ; publication et consommateurs externes peuvent les augmenter.

| Priorité / coût | Action | Risque, mesure et critère d'arrêt |
| --- | --- | --- |
| A — haut, 3–5 jours, 2–3 PR | Rendre runtime utilisable sans host/schema pour Rust pur en isolant [native_host](../../crates/argui-runtime/src/native_host.rs) et ses réexports derrière une capacité explicite ou une façade TSX. Vérifier app Rust pure, Solid/React natif et Web, automation. | Imports publics cassés, unification Cargo et cfg mobiles. Comparer cargo tree, cold/warm build et taille release de **la même app Rust** avant/après. Si gain négligeable, seulement clarifier les exports. |
| B — haut, 1–3 jours, 1–2 PR | Séparer la description d'EdgeFade/EdgeShadow du registre GPU pour que l'export du contrat schema compile sans render/wgpu ; préserver IDs et wire. | Valeurs ou comportement d'effet divergents. Comparer graphe et temps d'un petit outil schema isolé, snapshot JSON et rendu des effets. Runtime complet n'est pas le bon benchmark. |
| C — mesurer d'abord 0,5–1 jour ; corriger en 2–4 jours seulement si confirmé | Instrumenter Host::commit/materialize, décodage WireValue et publication des ThemeSnapshot. Réduire la ou les copies dominantes tout en gardant rollback et identité. | Changement d'ordre d'événements/atomicité. Comparer p50/p95, allocations, taille des patchs, géométrie et sortie visuelle ; arrêter sans signal clair. |
| D — moyen, 1–2 jours après mesure | Évaluer CLI léger pour init/check/add/update et chargement séparé de l'automation pour test/screenshot, sans modifier les commandes. | Deux binaires peuvent compliquer install/scripts. Mesurer taille CLI, compilation froide, démarrage et coût de distribution ; ne scinder que si le gain couvre le coût. |
| E — décision produit, 0,5 jour d'enquête | Décider si updater signé est une capacité d'application supportée distincte de argui update à SHA-256 ; documenter la promesse et tout éventuel plan de migration. | Perdre une API externe ou assimiler checksum et signature. Aucun retrait sans preuve de consommateurs, alternative et communication de version. |

L'ordre utile est de mesurer A et B sur leurs profils d'appels réels, puis C indépendamment. Les crates ui/layout/render/runtime sont grandes, mais leur réécriture complète n'a pas de justification empirique. Le critère de sortie est une expérience reproductible sur une app Rust pure et une app TSX, pas un quota arbitraire de crates.
