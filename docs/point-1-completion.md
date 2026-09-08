# Point 1 — preuves et exigences de clôture

> Mise à jour du 8 septembre 2026 : les passages et commandes concernant
> Shared Mailbox, Draft Studio et GalleryApplication ci-dessous sont historiques.
> Ces démonstrations et leurs tests dédiés ont été supprimés lors du recentrage
> de la galerie sur les widgets. Les contrats de modèles/montages restent testés
> dans `argui-runtime/tests/model/`. Le lanceur utilise `SingleWindowModel`.
> Les anciennes preuves de galerie ne sont plus des vérifications exécutables.


Audit du 7 septembre 2026, sur le worktree non commité. Périmètre :
« État partagé et cycle de vie » dans [les observations](oberservations.md),
lots A–D du [plan](shared-state-custom-elements-plan.md).
Ce document distingue une garantie absente, une preuve automatisée existante
et une intégration restant à vérifier. Il remplace, pour le statut du point 1,
les listes historiques contradictoires du suivi chronologique.

## 1. Ce que les tests prouvent déjà

Les noms ci-dessous désignent des tests existants, pas des tests à écrire.
Les résultats concernent les contrats exercés, pas toutes leurs intégrations OS.

| Garantie | Preuve existante | Portée de la preuve |
| --- | --- | --- |
| Données sans `Render`, executor partagé | `data_models_share_the_executor_without_rendering_or_parent_retention` dans [tasks.rs](../crates/argui-runtime/tests/tasks.rs) | Modèles et tâches sans présentation |
| Données partagées, caches/environnements/handlers distincts | `mounts_share_data_but_keep_caches_environments_and_event_owners_independent` dans [mount.rs](../crates/argui-runtime/tests/model/mount.rs) ; `window_adapters_retain_independent_mounts_of_one_shared_entity` dans [application.rs](../crates/argui-runtime/tests/application.rs) | Montages et adaptateurs headless, pas deux fenêtres OS |
| Observations locales, abonnements modèle survivant au démontage | `view_observation_invalidates_only_its_mount_and_ends_on_unmount`, `view_context_subscriptions_end_with_the_mount_but_model_subscriptions_survive` dans [events.rs](../crates/argui-runtime/tests/model/events.rs) | Isolation et durée de vie des abonnements |
| Annulation d'événements déjà en file | `mount_subscriptions_keep_their_environment_and_cancel_queued_delivery_on_close` dans events.rs | Aucun callback du montage fermé, livraison au montage survivant |
| Tâche locale annulée, tâche partagée conservée | `dropping_a_mount_cancels_its_task_but_not_shared_model_work` dans mount.rs ; `context_tasks_cancel_with_the_view_while_model_tasks_keep_running` dans [model/tasks.rs](../crates/argui-runtime/tests/model/tasks.rs) | Propriétaires entité/montage distincts |
| Résultats périmés et destruction du propriétaire | `dropping_a_mount_discards_its_already_queued_result_without_resurrection`, `mount_latest_replacement_and_closed_mount_reject_stale_work` dans model/tasks.rs | Résultats déjà prêts, remplacement et montage fermé |
| Fermeture idempotente et réentrante | `closing_a_retained_parent_releases_child_registrations_and_rejects_events`, `a_child_handler_can_close_its_parent_without_borrow_conflicts` dans mount.rs ; nettoyage réentrant/panique dans [scope.rs](../crates/argui-runtime/tests/model/scope.rs) | Ressources et handlers, pas notifications de cycle de vie |
| Bornes, ordre, réveils regroupés | Tests de [dispatch.rs](../crates/argui-runtime/tests/model/dispatch.rs), [cache.rs](../crates/argui-runtime/tests/model/cache.rs), [host.rs](../crates/argui-runtime/tests/model/host.rs) | Dispatcher, propagation et callbacks d'hôte ; pas certification de chaque boucle OS |
| Services typés et libération par portée | Les trois tests de [services.rs](../crates/argui-runtime/tests/model/services.rs) | Registre faible, domaines isolés, publication possédée par une `ResourceScope` |
| Arrêt explicite de l'executor | `explicit_shutdown_cancels_surviving_owners_and_refuses_new_work`, `cancellation_and_shutdown_release_all_scope_registrations` | `TaskRuntime::shutdown`, pas toute la séquence de sortie d'application |
| Exemple partagé, fermeture/réouverture et erreurs | tests application de la galerie (démonstration retirée), Shared Mailbox (démonstration retirée) | Commandes, adaptateurs, modèle et tâche de vue ; fenêtres OS non créées par ces tests |

Vérification relancée pendant cet audit :

```sh
cargo nextest run -p argui-runtime --all-features --test model --test tasks --test application
cargo nextest run -p argui-widget-gallery --all-features --test application --test pages -E 'binary(application) | test(shared_mailbox)'
```

Résultats : **98 tests runtime passés, aucun ignoré** ; **5 tests galerie passés**,
18 autres tests exclus par le filtre ciblé. Total : **103 tests exécutés et passés**.
Cela ne constitue ni un lancement du workspace entier ni une mesure de couverture.

## 2. Couverture : ce qui est réellement disponible

- Profil trouvé : `target/coverage/llvm-cov-target/argui.profdata`, dernière
  modification le **6 septembre 2026 à 16:17**, antérieure aux derniers changements.
- Aucun rapport de pourcentages à jour identifié dans les artefacts de couverture
  inspectés. La présence de fichiers instrumentés ne prouve ni le succès du gate
  ni la couverture du code actuel ; les anciens pourcentages ne sont pas réutilisés.
- [check-coverage.sh](../scripts/check-coverage.sh) exporte son résumé dans un fichier
  temporaire supprimé à la sortie. Le gate exige **90 % séparément** en lignes,
  fonctions, régions et branches. Il ne faut pas déduire un pourcentage du nombre
  de tests ni annoncer que le seuil est atteint sans nouvelle mesure.
- Le filtre du script exclut notamment `multi.rs`, `animation.rs` et plusieurs
  frontières d'hôte. Même un gate vert ne prouverait pas la fermeture des vraies
  fenêtres et le réveil de leurs boucles d'événements.
- Aucun nouveau lancement LLVM ni `quality.sh` pendant cet audit documentaire.
  Conserver un rapport daté et relié au commit/worktree, aux features et aux
  exclusions lors de la validation finale ; aucune couverture concurrente.

## 3. Exigences restantes, avec critères d'acceptation

Une case ouverte signifie « clôture non prouvée », pas nécessairement « code absent ».
Les garanties de la section 1 doivent être conservées, pas réimplémentées.

- [ ] **P1-01 — Visibilité explicite (contrat à ajouter).** Distinguer montage,
  vue masquée, fenêtre masquée/minimisée, démontage et destruction du modèle.
  Masquer conserve l'identité du montage, les données et les ressources ; les
  tâches ne sont pas annulées implicitement. Une éventuelle suspension est une
  politique explicite. Tester hide/show répété, mutation pendant masquage,
  absence de travail de rendu inutile et reprise avec les données courantes.
  Le booléen `Parent.visible` du test existant omet `cx.entity` : il teste un
  **retrait du rendu**, pas ce contrat de visibilité.
- [ ] **P1-02 — Notifications de cycle de vie (contrat à ajouter).** Exposer les
  événements de montage, changement de visibilité et démontage, ainsi que la
  politique de destruction. Fixer l'ordre parent/enfant et la livraison unique
  aux frontières de transaction, hors emprunts actifs et phases layout/paint.
  Tester notification qui mute un modèle, ferme un parent ou demande un remontage,
  fermeture répétée et création interrompue. Documenter la politique en cas de
  panique sans empêcher la libération des autres ressources.
- [ ] **P1-03 — Ownership des hôtes (intégration à prouver).** Établir une matrice
  application/fenêtre/montage/entité/service/tâche avec créateur, propriétaire,
  événement de fermeture et survivants autorisés. `ResourceScope` et l'arrêt de
  `MultiApplication` existent ; vérifier tous les chemins mono-fenêtre,
  multi-fenêtres, GTK et Web, y compris échec d'ouverture et sortie avec handles
  externes retenus. Aucun résultat de tâche ni événement en file ne ressuscite
  une présentation fermée. Définir aussi ce qui reste utilisable sur un modèle
  conservé après l'arrêt de son application.
- [ ] **P1-04 — Séquence complète à deux fenêtres (preuve à compléter).** Modèle
  et service communs, tâches de modèle et de vue simultanées ; fermer une fenêtre
  pendant des livraisons, continuer dans l'autre, rouvrir puis quitter. Vérifier
  nouvelle identité de montage, rejet des anciens handlers/résultats, conservation
  du modèle et des services jusqu'à leur vrai propriétaire. Éprouver l'arrêt de
  l'application avec une tâche terminée mais pas encore livrée. Ne pas se limiter
  à appeler directement `TaskRuntime::shutdown` dans ce test d'intégration.
- [ ] **P1-05 — Réveils réels (preuve à compléter).** Après une mutation/tâche,
  les seules fenêtres dépendantes sont actualisées sans clic, resize ni animation.
  Saturer le budget pour vérifier la reprise au tour suivant, fermer une fenêtre
  entre notification et livraison, puis vérifier le repos sans polling. Inclure
  modèles de services non rendus et fenêtres cachées selon P1-01, natif et Web.
- [ ] **P1-06 — Frontières API et migrations (audit à terminer).** Conserver la
  séparation `ModelContext`/`Context` déjà présente. Ajouter des tests de compilation
  prouvant qu'un contexte modèle ne permet pas le focus/layout de fenêtre. Vérifier
  les consommateurs hôtes, DevTools, wrappers de sélection et galerie ; supprimer
  seulement les anciens chemins réellement identifiés, sans couche de compatibilité.
- [ ] **P1-07 — Stress et repos (mesure à compléter).** Faire 1 000 cycles de
  montage/démontage/remontage d'une vue avec abonnements, tâches et services ;
  ajouter hide/show et fermeture/réouverture. Compter les ressources vivantes,
  handlers, abonnements et tâches après drainage ; retour au niveau initial.
  Les 1 000 annulations de `scope.rs` ne remplacent pas ce scénario combiné.
  Mesurer idle et diffusion à plusieurs vues, noter machine/protocole et distinguer
  rétention de l'allocateur d'une fuite. Aucun seuil de FPS inventé.
- [ ] **P1-08 — Démonstration et clôture.** Étendre Shared Mailbox pour montrer
  séparément masquer/afficher et démonter/remonter, avec identités et compteurs
  réels, pas des statuts simulés. Relier chaque P1 à ses tests et résultats ;
  actualiser observations/documentation, vérifier natif/Web, puis exécuter
  `quality.sh` une seule fois à la fin, immédiatement avant un commit autorisé.
  Un gate échoué ou une vérification non exécutée reste explicitement ouvert.

Ordre conseillé : P1-01/P1-02, P1-03/P1-06, P1-04/P1-05, P1-07/P1-08.
Ne pas confier à l'utilisateur la preuve de l'absence de fuites ou de callbacks
périmés : ces propriétés demandent des tests et compteurs automatisés.

## 4. Vérifications manuelles utilisateur

### Possible dès maintenant

Lancer `cargo run -p argui-widget-gallery --all-features`, puis **Examples →
Shared Mailbox** :

1. Ouvrir **Open native mailbox window**. Choisir des messages différents dans
   les deux fenêtres et scroller une seule liste : sélection et scroll indépendants.
2. Marquer un message lu/non lu : les données et compteurs concernés se mettent
   à jour dans les deux fenêtres, sans clic supplémentaire dans la seconde.
3. Fermer la fenêtre auxiliaire avec sa croix : la principale reste utilisable.
   La rouvrir : mêmes données partagées, nouvelle présentation locale.
4. Lancer **Count all subjects · model-owned**, puis **Unmount right view** :
   la tâche du modèle doit aboutir sans interaction supplémentaire. Une tâche
   **Count subjects · view-owned** encore en cours doit être annulée au démontage.
   Si elle finit avant le clic, cela ne teste pas l'annulation : les tests
   automatisés contrôlent ce cas sans dépendre de la vitesse du clic.
5. Tester clavier/focus et tailles différentes ; si possible, déplacer une fenêtre
   vers un écran de DPI différent. Vérifier absence de décalage des clics et que
   le focus ne modifie pas la sélection de l'autre fenêtre.

Sur Web : `./scripts/serve-widget-gallery.sh`, URL **`/widgets/`**, même page ;
vérifier les deux vues intégrées, démontage/remontage et résultats sans clic.
Le bouton de fenêtre native doit annoncer son indisponibilité, pas ouvrir une
fausse fenêtre. Réduire puis réactiver l'onglet vérifie la reprise du navigateur,
pas à lui seul le futur contrat P1-01.

Noter OS, backend (Wayland/X11/etc.), navigateur, DPI, scénario et résultat ;
joindre console/logs seulement en cas d'erreur. Les déclarations utilisateur
antérieures « ça marche » ne sont pas converties en validation de tous ces cas.

### Masquage de présentation désormais disponible

Dans Shared Mailbox, **Hide right view / Show right view** conserve le montage.
Son identifiant **Mount** doit rester identique après réaffichage, y compris après
une mutation depuis la vue gauche. **Unmount right view / Remount right view**
crée en revanche une nouvelle identité. La validation des notifications de cycle
de vie reste à effectuer après P1-02.

## 5. Implémentation en cours — 7 septembre 2026

Les cases P1 restent ouvertes tant que l'ensemble de leurs critères n'est pas
prouvé. Les ajouts suivants sont présents dans le worktree, sans commit ni gate
qualité final :

- **P1-01, partie présentation** : `Mount::set_visible`, `Mount::is_visible` et
  `Context::entity_visible` distinguent masquage et démontage. Le montage masqué
  conserve identité, ressources, abonnements et tâches ; rendu, layout, frames et
  événements UI sont suspendus. Les mises à jour de tâches masquées ne demandent
  pas de frame au parent. Reprise avec les données courantes et refus de montrer
  un montage fermé : preuves dans
  [visibility.rs](../crates/argui-runtime/tests/model/visibility.rs).
  **Restant :** visibilité/minimisation de fenêtre et intégrations des hôtes.
- **P1-03, livraison tardive** : le chemin `tasks_ready` rejette désormais les
  présentations fermées. Le test `closing_the_model_prevents_late_host_task_callbacks`
  conserve l'adaptateur et le modèle après fermeture de ses ressources.
- **P1-07, stress combiné** :
  `a_thousand_visibility_close_and_reopen_cycles_release_all_owned_work` effectue
  1 000 cycles avec deux montages, handlers capturants, abonnements, tâches et
  service partagé, puis hide/show, fermeture pendant livraison et réouverture.
  Chaque cycle vérifie zéro ressource restante, zéro événement/invalidation,
  annulation des tâches, disparition du service et du montage faible, retour des
  références capturées au niveau initial, et rejet des anciens handlers.
  **Restant :** mesures idle/diffusion et mémoire avec protocole/machine documentés ;
  ce test n'est pas une mesure de mémoire du processus ni une preuve OS.
- **P1-08, démonstration** : deux commandes distinctes de visibilité et de montage
  dans Shared Mailbox ; identité réelle affichée via `Context::mount_id`.
  `mailbox_visibility_preserves_mount_identity_while_remount_changes_it` vérifie
  les contrôles, une mutation pendant masquage et les identités.
  **Restant :** compteurs et notifications, démonstration native/Web et gate final.

Validation ciblée de cette étape, features inchangées :

```sh
cargo nextest run -p argui-runtime --all-features --test model --test tasks --test application
cargo nextest run -p argui-runtime --all-features --test model -E 'test(visibility)'
cargo nextest run -p argui-widget-gallery --all-features --test pages -E 'test(shared_mailbox)'
```

Le premier run a exécuté **103 tests passés**, dont le stress combiné (0,130 s
rapportées par Nextest pour ce test, sans seuil de performance). Le filtre visibilité a ensuite exécuté **6 tests passés**, dont le test
supplémentaire de fermeture ; le filtre Shared Mailbox a exécuté **4 tests passés**
(18 autres exclus). Ces runs se recouvrent ; leurs nombres ne sont pas additionnés.
Aucune couverture LLVM ni exécution de `quality.sh` à cette étape.

### Frontière de compilation P1-06

Les contrats de [model_context.md](../crates/argui-runtime/tests/model/model_context.md)
sont inclus dans la documentation publique de `ModelContext`. Trois compilations
négatives couvrent `request_focus`, `observe_bounds` et `environment` sur un modèle
qui implémente lui-même `Render`. Un consommateur positif vérifie ces mêmes
capacités sur un montage vivant, ainsi que la mutation d'un modèle sans `Render`.

```sh
cargo test -p argui-runtime --all-features --doc model::model_context::ModelContext
```

Résultat du 7 septembre 2026 : **4 contrats passés** (3 rejets de compilation,
1 consommateur valide). Cette vérification est désormais intégrée à `quality.sh`,
après le check Web et avant LLVM. Elle utilise Rustdoc pour la compilation ;
les tests de comportement restent exécutés par Nextest. Le gate complet n'a pas
été lancé.

L'audit a confirmé que `SelectionHost::render` utilise `cx.entity` et transmet
le layout avec `cx.layout_entity`, comme l'implémentation `Render` de
`DevtoolsHost`. Le chemin public distinct `DevtoolsHost::view`, qui appelait
`self.app.render()` directement avec un environnement par défaut, a été supprimé.
Ses consommateurs (tests de rendu, style, hôte et exemple de profilage) utilisent
maintenant des `Mount<DevtoolsHost<_>>` et leur rendu conservé. Les mutations de
fixtures notifient explicitement le montage ; la publication d'un instantané
externe n'est plus confondue avec l'invalidation automatique du rendu.

Les tests de navigation et d'instantanés de l'arbre sont regroupés dans
[host/tree.rs](../crates/argui-devtools/tests/host/tree.rs), en miroir de la
responsabilité source existante. Aucun fichier Rust ne dépasse 600 lignes.

```sh
cargo nextest run -p argui-devtools --all-features
cargo clippy -p argui-devtools --all-targets --all-features -- -D warnings
```

Résultat du 7 septembre 2026 : **44 tests DevTools passés, aucun ignoré** ;
Clippy passe, y compris l'exemple migré. Les contrôles de taille, miroir des tests
et hygiène passent. Aucun appel `.view()` sans arguments ni déclaration du chemin
supprimé ne subsiste dans la crate DevTools. L'API `AppModel::view(window,
environment)` demeure le contrat applicatif multi-fenêtres, distinct du chemin
supprimé. La vérification finale des hôtes, de la galerie et du Web reste couplée
aux exigences P1-03/P1-05/P1-08 ; P1-06 n'est pas encore coché globalement.

### Fermeture réentrante et matrice P1-03

La [matrice d'ownership](ownership.md) décrit pour chaque ressource son créateur,
son propriétaire fort, sa fermeture et les survivants autorisés. Elle distingue
la lecture des chemins sources de la preuve d'exécution OS encore requise, et
précise les capacités des modèles conservés après arrêt de leur application.

Deux défauts supplémentaires ont été corrigés : la traversée des frames libère
son emprunt de la liste des enfants avant les callbacks, et un rendu interrompu
par la fermeture du montage n'installe plus son sous-arbre ni ses nouveaux handlers.
Les tests `a_child_frame_can_close_its_parent_and_prevent_remaining_frame_callbacks`
et `closing_during_render_does_not_retain_new_handlers_or_publish_a_subtree` dans
[mount.rs](../crates/argui-runtime/tests/model/mount.rs) vérifient ces garanties,
le rejet des frames du parent/frère fermé et la libération des captures.

```sh
cargo nextest run -p argui-runtime --all-features --test model -E 'test(mount)'
```

Résultat : **19 tests passés, 60 exclus par le filtre**, le 7 septembre 2026.
Ces preuves ne remplacent pas les notifications de cycle de vie P1-02 ni les
séquences d'ouverture/échec/sortie natives et Web P1-03/P1-04.

### Notifications P1-02 — contrat runtime ajouté

`ModelRuntime::observe_mounts` expose les transitions de montage, visibilité et
démontage avec les identifiants réels. Le [contrat de livraison](lifecycle.md)
précise l'ordre parent/enfant, l'annulation, la destruction des données, les
panics et le drainage au réveil de l'hôte. Les notifications ne s'exécutent pas
à la sortie d'une transaction de rendu : elles attendent un appel explicite à
`dispatch_pending`, hors phases de présentation, y compris entre domaines modèles.

Les [huit tests de cycle de vie](../crates/argui-runtime/tests/model/lifecycle.rs)
couvrent les transitions uniques, mutation depuis notification, fermeture et
remontage depuis callback, annulation des livraisons, ordre montage/démontage,
création interrompue, panique et reprise des autres observateurs, budget de 64
étapes et retour au repos. Les notifications ne retiennent pas le modèle.

Validation du 7 septembre 2026 : `cargo nextest run -p argui-runtime --all-features
--test model` a passé **86 tests** avant le dernier cas de création interrompue ;
le filtre `-E 'test(lifecycle)'` a ensuite passé **8 tests**, dont ce dernier cas
(79 autres exclus). Clippy runtime et les contrôles structurels passent.

P1-02 reste ouvert pour la livraison en fermeture/sortie des vrais hôtes, la
propagation de visibilité fenêtre/onglet et la démonstration. Le budget de dispatch
borne le travail par tour ; il ne borne pas encore le nombre de transitions
produites avant drainage. Aucun gate qualité final ni mesure LLVM exécuté ici.

### Compteurs de démonstration P1-08

Shared Mailbox possède désormais un abonnement réel à `observe_mounts`, filtré
sur le modèle de la vue droite. La ligne `Right view` affiche les montages vivants
et les nombres de transitions de montage, masquage, réaffichage et démontage.
Ces valeurs changent à la livraison des événements du runtime, pas dans les
handlers des boutons. Le callback ne retient qu'un handle faible vers la démo.
Les identifiants des montages terminés sont retirés de l'ensemble des montages
vivants ; aucun journal croissant d'identifiants historiques n'est conservé.

`Mount::dispatch_models` permet aux hôtes personnalisés/headless de drainer un
lot borné par domaine modèle attaché, au réveil et hors rendu. Le test galerie
`mailbox_displays_delivered_lifecycle_counts_without_simulated_transitions`
l'utilise pour vérifier les compteurs après montage, hide/show, démontage et
remontage. Le montage masqué reste vivant ; le remontage augmente le compteur
de création et reçoit une nouvelle identité (test d'identité existant).

```sh
cargo nextest run -p argui-widget-gallery --all-features --test pages -E 'test(shared_mailbox)'
cargo clippy -p argui-runtime -p argui-widget-gallery --all-targets --all-features -- -D warnings
```

Résultat du 7 septembre 2026 : **5 tests passés, 18 exclus**, Clippy et contrôles
structurels passés. La preuve reste headless : les véritables réveils natifs/Web,
la fermeture de l'application et le gate final restent ouverts.

### Vérification navigateur et défaut de repos P1-05/P1-07

Retour utilisateur : hide/show conserve les identifiants, les messages se mettent
à jour en temps réel, le remontage change l'identifiant et la fenêtre native
fonctionne. Ce retour confirme ces interactions, sans certifier les scénarios
internes de fermeture, mémoire ou callbacks périmés.

Le scénario Shared Mailbox Web (démonstration retirée)
a été étendu et exécuté dans **Chrome 152.0.7977.54**, headless avec SwiftShader,
sur Linux 7.1.12-200.fc44.x86_64, Intel Core Ultra 5 125H (18 processeurs logiques),
viewport 1220 × 1000. Le serveur est construit depuis le worktree via
`./scripts/serve-widget-gallery.sh 8793` (Wasm dev, feature `webview`).

Les tâches livrées sans interaction supplémentaire, les compteurs de notifications,
les mutations pendant trois cycles hide/show, la conservation/remplacement des
identifiants et l'indisponibilité de fenêtre native sur Web ont passé.

L’assertion de repos a révélé deux défauts, désormais corrigés : la galerie
montait les démonstrations des pages non sélectionnées, dont le spinner actif ;
une mutation de modèle entièrement propagée pendant sa transaction pouvait ne
plus réveiller l’hôte. La galerie monte désormais uniquement la page sélectionnée,
et une mutation notifiée demande un réveil coalescé même sans livraison restante.
Les modèles des autres pages restent conservés ; leurs présentations sont démontées.

Après correction : **0 demande de `requestAnimationFrame` pendant 1 000 ms**,
après 1 000 ms de stabilisation, et scénario Web complet passé. Cette mesure
porte sur les demandes de frame, pas sur la consommation CPU ou la mémoire.
Les sondes temporaires ont été retirées.

Régressions automatisées :
- `gallery_returns_to_idle_after_mailbox_navigation` alterne trois fois entre la
  page animée et Shared Mailbox, puis vérifie aussi le wrapper DevTools.
- `completed_model_invalidation_still_wakes_an_idle_host_once` vérifie deux séries
  de mutations, leur coalescence, la reprise après acquittement et le repos après
  drainage, sans animation ni clic auxiliaire.

Nextest avec `--all-features` : **88 tests modèle runtime et 42 tests galerie
passés, aucun ignoré**. Les preuves natives de fermeture complète et le gate final
restent ouverts ; ce résultat ne clôture pas à lui seul le point 1.

Commande utilisée (Puppeteer installé hors du dépôt, sans dépendance Cargo ajoutée) :

```sh
PUPPETEER_MODULE=/tmp/argui-browser/node_modules/puppeteer-core/lib/puppeteer/puppeteer-core.js \
CHROME_PATH=/home/albi/.cargo/bin/google-chrome \
node crates/argui-widget-gallery/tests/pages/shared_mailbox.mjs
```
