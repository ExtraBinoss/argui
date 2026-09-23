# Passation — plan UI générale et performance DSL

État au 23 septembre 2026. Le plan de référence est
[GENERAL_UI_PLAN.md](GENERAL_UI_PLAN.md), sa liste de critères est
[GENERAL_UI_CHECKLIST.md](GENERAL_UI_CHECKLIST.md), et les reproductions
initiales sont dans [GENERAL_UI_BASELINE.md](GENERAL_UI_BASELINE.md).
Ce chantier part du commit `d22fb04` de `codex/dsl-gallery-live`. Le commit de
passation est destiné à cette même branche. **Le plan P0–P10 n'est pas terminé.**
Les cases non cochées de la checklist restent ouvertes, même quand une partie
de leur comportement a été implémentée.

**Action Git pour la prochaine conversation :** travailler dans le checkout de
`codex/dsl-gallery-live`, puis fusionner la branche locale propre
`codex/gallery-text-fidelity` (actuellement `2968031`) dans cette branche.
Résoudre les éventuels conflits et vérifier les zones touchées avant de
poursuivre le plan. Cette fusion text-fidelity **n'est pas encore faite** ; les
mesures utilisateur ci-dessous portent sur la galerie avant cette fusion.

## État par phase

| Phase | État réel | Travail restant principal |
| --- | --- | --- |
| P0 | Baseline établie, non bloquant | L'inventaire et quelques preuves administratives de la checklist restent ouverts ; ne pas relancer l'audit pour avancer. |
| P1 | Correctifs principaux implémentés, non bloquant | Validation sémantique des écritures, directions, types, clés de repeater, court-circuit, arithmétique contrôlée et warnings CLI/LSP ajoutés. Les 18 fixtures de conformité AOT/live passent. Ne revenir sur les cas limites que si un défaut concret apparaît. |
| P2 | Partiel | Slots nommés, défauts, forwarding et styles/états fonctionnels sur les exemples couverts ; Dialog et Input adaptés. Restent cardinalité, paramètres typés des templates, références lexicales générales par instance et enfant conditionnel absent, ainsi que les preuves complètes de précédence. |
| P3 | À faire | Grid, layout général, propriétés visuelles, transformations, hit testing et preuves responsive. |
| P4 | À faire | Expressions et modèles réactifs mutables, hauteurs variables, maintien du focus/scroll et mesures. |
| P5 | À faire | Façade Rust typée commune, services asynchrones et extensions natives. |
| P6 | À faire | Dessin vectoriel réactif et gestes. |
| P7 | À faire | Accessibilité complète, texte riche et internationalisation/RTL. |
| P8 | Partiel | La suppression d'une définition d'enfant monté lors d'un reload est corrigée. Restent les autres changements structurels, l'identité/état, le focus et l'outillage. |
| P9 | À faire | Trois applications de preuve AOT/live, matrice galerie et mesures. |
| P10 | À faire | Revue finale, plateformes, gate qualité et couverture. |

## Performance et animations : ce qui est connu

- Les 14 pistes d'Animation Lab changent sur plusieurs cycles dans le test
  headless AOT et live ; pause/reprise est aussi vérifiée. Le runtime conserve
  la phase des motions lors de la pause. L'utilisateur confirme aussi que
  toutes les animations fonctionnent sur son bureau.
- La sidebar DSL ne monte plus que les lignes visibles via `VirtualWindow` ;
  les petites impulsions de molette et le thumb ont été vus sur l'affichage
  privé. Le cache des éléments natifs, la surveillance ciblée des hauteurs,
  le partage de l'IR live, les références d'enfants précalculées et le chemin
  rapide des valeurs littérales réduisent les recalculs identifiés.
- **Mesures manuelles communiquées par l'utilisateur sur son bureau :** avec
  `cargo run --manifest-path app_examples/Cargo.toml -p argui-example-widget-gallery-dsl`,
  Button consomme environ **0,8 % CPU**, Animation Lab **1,5 % maximum** et
  **0 % quand aucune animation ne joue**, Damage Control environ **1,1 %**.
  Toutes les animations tournent. Avec `argui dev` (commande ci-dessous), il
  ne voit presque aucune différence CPU : **0,1–0,2 point** de marge observée.
  Ces chiffres sont des observations utilisateur, pas un benchmark automatisé.
  Ils lèvent le blocage pratique de performance signalé pour avancer dans le
  plan ; vérifier de nouveau après la fusion text-fidelity ou une régression.
- Les anciennes mesures de l'affichage privé restent **non concluantes** :
  après quelques frames, wgpu retourne `RenderStatus::Skipped` et l'image se
  fige, y compris pour le spinner de la galerie Rust native. Ne pas confondre
  ce défaut de l'environnement de test avec les résultats sur le bureau de
  l'utilisateur.

## Commandes de reproduction

Depuis la racine de ce dépôt, lancer la galerie DSL compilée :

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-widget-gallery-dsl
```

Pour comparer avec la galerie Rust native :

```sh
cargo run -p argui-widget-gallery --all-features
```

Commande dev utilisée par l'utilisateur depuis
`app_examples/widget-gallery-dsl` ; elle démarre le client automatiquement :

```sh
cd app_examples/widget-gallery-dsl
cargo run --manifest-path ../../Cargo.toml -p argui-cli --bin argui -- dev
```

Pour lancer séparément le client live, utiliser deux terminaux, le premier
depuis `app_examples/widget-gallery-dsl`, le second depuis la racine :

```sh
cargo run --manifest-path ../../Cargo.toml -p argui-cli --bin argui -- dev ui/main.argui 127.0.0.1:4777 --no-run
```

```sh
ARGUI_DEV_ADDRESS=127.0.0.1:4777 cargo run --manifest-path app_examples/Cargo.toml -p argui-example-widget-gallery-dsl --features argui-live
```

En cas de régression, faire défiler la sidebar, laisser Animation Lab et
Damage Control tourner sans interaction, utiliser Pause/Resume et comparer les
mesures avec les chiffres utilisateur ci-dessus. Signaler aussi si le clic
d'une page de la sidebar n'apparaît qu'après un scroll : ce symptôme n'a pas
été isolé de la surface privée qui saute les présentations.

## Priorités pour la prochaine conversation

1. Se placer sur `codex/dsl-gallery-live`, fusionner
   `codex/gallery-text-fidelity` et traiter les conflits éventuels. Vérifier
   ensuite les parties concernées par cette fusion.
2. Prendre les mesures utilisateur ci-dessus comme nouveau point de référence.
   La performance ne bloque plus la suite ; ne rouvrir l'enquête CPU que si
   une régression apparaît, notamment après la fusion.
3. Ne pas relancer un audit ou une campagne de commandes pour P0 : sa baseline
   existe déjà. Les correctifs principaux de P1 sont implémentés et les 18
   scénarios AOT/live passent ; ne rouvrir P1 que pour un défaut concret ou
   une dépendance rencontrée. Compléter les vrais manques de P2 (cardinalité
   des slots, paramètres typés des templates, références lexicales par
   instance, enfant conditionnel absent), puis avancer sur P3–P10.
4. Mettre à jour les cases et le registre de preuves seulement après validation.
   Appliquer `docs/contributing/code-quality.md` et
   `docs/contributing/linux-testing.md`. Les contrôles GUI de l'agent vont
   sur l'affichage privé, avec inspection de captures non blanches.
5. Faire le gate global et la couverture seulement quand P10 est vraiment
   prêt. `./scripts/quality.sh` n'a pas été exécuté dans cette passation à la
   demande de l'utilisateur d'arrêter les tests ; ne pas déclarer 85 % acquis.

## Tests et vérifications déjà effectués

- Conformité DSL : 18/18 scénarios ciblés AOT et live passés, dont slots,
  styles, clés, affectations et court-circuit.
- Après virtualisation de la sidebar : 3/3 tests ciblés de galerie passés,
  dont boucle/pause des 14 pistes d'Animation Lab en AOT/live et parité
  visuelle (`/tmp/argui-sidebar-final-focused.log`).
- Schéma `VirtualWindow` et scrollbar explicite : 5/5 tests ciblés passés
  (`/tmp/argui-virtual-scrollbar-test.log`).
- Passe antérieure runtime/compiler/protocole : 203/204, avec une assertion
  devenue obsolète sur le texte Rust généré ; cette assertion corrigée et le
  nouveau test de binding ont ensuite passé 2/2. La régression entière reste
  à rejouer.
- Passe antérieure de galerie : 25/26 ; le test de boucle échouait sur la fin
  d'une transition de navigation, corrigée puis vérifiée par le 3/3 ci-dessus.
- Contrôle structurel : aucun fichier Rust de `crates/` ou `app_examples/`
  ne dépasse 600 lignes après extraction du test responsive.
- Les mesures CPU sur écran normal rapportées par l'utilisateur figurent
  ci-dessus ; aucun gate global ni couverture LLVM n'est validé. La passe de
  galerie lancée juste avant la demande d'arrêt des tests n'est pas utilisée
  comme preuve de validation finale.
