# Argui : moteur natif commun pour Solid et React

**Décision proposée — 23 septembre 2026.** Ce plan remplace la progression P1–P10
du langage `.argui`, arrêtée à P4. Il porte sur une refonte, pas sur une couche
de compatibilité. L'audit du DSL a été fait sur `codex/dsl-gallery-live` ; ce
document est préparé dans le worktree courant pour relecture avant exécution.

## Résultat recherché

- Solid est la première façon d'écrire une application Argui et ses widgets, en
  TSX avec `solid-js/universal`. React pourra utiliser le même moteur par son
  propre adaptateur et ses propres composants.
- `Rectangle`, `Text`, `Row`, `Column`, `Grid`, `TextInput`, `Image`, etc.
  restent des primitives **natives Argui**, décrites une fois dans le schéma
  Rust. Aucun DOM ou WebView n'intervient dans leur rendu desktop/mobile.
- Les deux adaptateurs produisent **la même représentation native** : un seul
  `UiTree`, avec les mêmes règles d'identité, de layout, de texte, d'input, de
  peinture, d'animation et d'accessibilité. Les objets temporaires détenus par
  Solid et React servent uniquement à leurs cycles de vie et au groupement des
  changements.
- La première application Solid inclut un **shell complet** : topbar, sidebar
  de navigation et sélection de page, ainsi que thème clair/sombre et accents.
  Elle ne référence que les pages réellement implémentées.
- Le langage `.argui`, ses widgets, son compilateur, son runtime live, son LSP,
  son CLI spécialisé, ses exemples, ses tests et sa documentation sont supprimés
  entièrement. Pas de parseur dormant, de façade vide, de fichiers `.argui`, ni
  de code de compatibilité.
- Le choix d'un moteur JavaScript ne change ni le contrat de l'arbre Argui ni
  les composants TSX. Il change l'adaptateur d'hébergement et le packaging.

**Contrat de parité.** Une primitive donnée reçoit les mêmes propriétés typées
et les mêmes événements dans Solid et React, puis produit le même comportement
natif. Les widgets composés peuvent être différents dans les deux bibliothèques ;
leur identité visuelle exacte n'est pas une propriété implicite du moteur.

## État du dépôt utile à la décision

| Constat vérifié dans `codex/dsl-gallery-live` | Décision |
| --- | --- |
| `argui-ui::UiTree` réconcilie des `Element` partagés par `Rc` et conserve focus, scroll, texte et interactions. Des tests prouvent qu'un sous-arbre inchangé de 10 000 éléments est ignoré. | Garder cet arbre comme état natif canonique ; mesurer les mutations JS avant toute refonte de sa structure. |
| `argui-schema` définit déjà `Rectangle`, `Text` et les autres primitives, leurs propriétés, événements, slots et constructeurs natifs. | Garder et rendre ce registre indépendant de tout langage ; en tirer les types TSX. |
| `argui-accessibility` possède des rôles, valeurs, états, actions et relations ; le runtime publie des mises à jour AccessKit sur les plateformes natives et un arbre sémantique sur le web. | Réutiliser ce modèle ; compléter l'exposition et les tests, notamment sur mobile. |
| La bibliothèque DSL contient 57 fichiers `.argui` ; `argui-widgets` contient également des widgets Rust. | Reprendre d'abord les comportements utiles à Button et Select, puis l'Animation Lab comme démonstrateur TSX. Reporter le reste du catalogue. |
| Android utilise `NativeActivity` ; iOS lance le runtime Rust dans une fenêtre UIKit gérée par winit. Le Swift actuel sert surtout aux intégrations système, dont ActivityKit. | Garder le rendu Argui et les services Java/Swift. Vérifier tôt l'hébergement JS et l'accessibilité native sur chaque plateforme. |

Ces constats viennent de `crates/argui-ui/src/tree.rs`,
`crates/argui-ui/tests/tree.rs`, `crates/argui-schema/src/builtin.rs`,
`crates/argui-accessibility/src/schema.rs`, `crates/argui-runtime/src/app/accessibility.rs`,
`crates/argui-android/src/lib.rs` et `crates/argui-ios/src/lib.rs`.
Ils ne constituent **pas** une validation de performance Solid/React ni un test
sur appareil mobile.

## Architecture cible

```text
@argui/widgets-solid       @argui/widgets-react
        │                           │
@argui/solid (Universal)   @argui/react (reconciler)
        └─────────────┬─────────────┘
                      │
          @argui/host : IDs, événements,
          mutations groupées, types générés
                      │
             pont JS/Rust de l'hôte choisi
                      │
          argui-host : validation + commit
                      │
     argui-schema → Element → UiTree → layout/paint/GPU
                      │
       arbre sémantique → adaptateur accessibilité OS
```

Le cœur Rust garde `Render`/`AppModel` pour ses propres clients, mais les
changements JS entrent par un hôte de transactions. Solid et React ne passent
jamais par `dsl-ir`. Une bibliothèque JavaScript n'implémente ni la disposition
des éléments, ni le focus, ni l'édition de texte : ces responsabilités restent
dans Argui.

### Contrat minimal de l'hôte

Une transaction contient des opérations typées : `create(id, nativeType)`,
`setProperty(id, propertyId, value)`, `setListener(id, eventId, callbackId)`,
`insert(parent, child, before?)`, `remove(id)` et `setRoot(id)`. `commit(batch)`
valide IDs, générations, types, ordre des enfants et absence de cycles **avant**
d'altérer l'arbre visible. Un batch invalide laisse l'ancienne UI intacte.

- `HostId` est stable durant le montage JS ; `NodeId` et l'état interactif restent
  possédés par `UiTree`. Les démontages libèrent listeners, références JS et
  ressources natives. Déplacement et réordonnancement conservent l'identité.
- Les adaptateurs conservent un graphe JS minimal pour les méthodes synchrones
  demandées par Solid/React et les sous-arbres non encore commis. Une mise à
  jour acceptée traverse la frontière JS/Rust **en un lot**, pas un appel FFI
  par propriété. Solid vide son lot au plus tard avant la prochaine frame et
  avant une lecture de layout ; React n'envoie que le travail réellement commis.
- `UiTree` et le GPU restent sur le thread UI. Le pont poste les lots vers ce
  thread et renvoie événements/résultats vers le thread JS ; aucune attente
  bloquante circulaire n'est autorisée. Les callbacks d'un nœud démonté sont
  invalidés par sa génération.
- Le Rust reconstruit au départ seulement les chemins modifiés en partageant
  les autres `Element`, puis appelle `UiTree::update`. Ses statistiques
  (`visited`, sous-arbres partagés, type d'invalidation) déterminent si ce
  chemin suffit. Si de grands groupes de frères imposent trop de visites,
  ajouter des mutations ciblées à `UiTree` sur la base d'un profil mesuré.
- Le sens inverse transporte `callbackId`, événement et cible vers JS. Les
  événements clavier, clic et composition IME gardent leur ordre ; les seuls
  événements éventuellement regroupés sont ceux dont la sémantique le permet
  (par exemple certains déplacements du pointeur).
- Les mesures de géométrie lisent le dernier layout validé après commit/frame ;
  aucune lecture synchrone depuis JS ne bloque la boucle UI en attendant une
  frame qu'elle doit elle-même produire. Les animations natives continuent
  sans exécuter un signal JS à chaque image.
- Le schéma Rust est la source unique des noms, types, valeurs par défaut,
  événements et capacités. Un outil génère les types JSX et vérifie un hash de
  contrat ; aucune seconde liste de primitives tenue à la main en TypeScript.

`napi-rs` convient à un hôte Bun desktop ; `deno_core` peut appeler directement
des opérations Rust depuis V8 ; QuickJS passe par son API C. Ces ponts restent
des **implémentations** : `argui-host`, les opérations et les widgets ne
dépendent d'aucun des trois.

## Solid d'abord, React sur le même hôte

1. Compiler JSX pour la cible *universal* de Solid et implémenter exactement
   les méthodes nécessaires (`createElement`, `createTextNode`, `setProperty`,
   `insertNode`, `removeNode`, navigation parent/enfants et remplacement du
   texte). Fournir `jsxImportSource` et des types natifs. Épingler une version
   stable de Solid ; évaluer Solid 2 séparément de la première livraison.
2. Exposer des composants typés `Rectangle`, `Text`, `Row`, `Column`, etc. qui
   aboutissent au registre natif. Implémenter d'abord `Button` et `Select` en
   Solid TSX, avec les comportements natifs Argui appropriés. Recréer
   l'**Animation Lab** en TSX pour éprouver les animations du moteur.
3. Une fois le contrat testé avec Solid, ajouter `@argui/react` via
   `react-reconciler`. Créer les nœuds de travail hors de l'arbre visible et
   n'envoyer que les mutations du commit accepté. Épingler la version du
   reconciler et maintenir un test d'upgrade : son API est expérimentale.
4. Les tests communs de l'hôte et des primitives s'exécutent sur les deux
   adaptateurs. Les tests de Button, Select et Animation Lab comparent aussi
   leurs résultats natifs ; les tests propres à chaque composant restent dans
   sa bibliothèque.

### Portée initiale des composants

Ne **pas** réécrire tout le catalogue de widgets ni toutes les pages de la
galerie DSL au début. Les deux premiers widgets publics Solid sont **Button**
et **Select** ; **Animation Lab** est une page de démonstration et de mesure,
pas un widget de la bibliothèque. Ce trio suffit à éprouver les interactions,
le focus, les popups, la sélection, les états visuels et les animations avant
d'élargir le catalogue.

Le **shell de la galerie** fait partie de cette première tranche : reconstruire
le sidebar et la topbar en Solid, avec navigation entre Button, Select et
Animation Lab, indication accessible de la page courante et présentation
adaptée aux petites fenêtres. Reprendre le système de thème existant avec
clair/sombre et accents bleu/violet/émeraude, en gardant les tokens natifs
partagés. Changer de page ou de thème doit conserver l'état non concerné et
ne pas remonter toute l'application. Le sidebar ne montre aucune ancienne page
dont le contenu `.argui` a été supprimé ; ajouter les entrées au fur et à
mesure des futures implémentations. Ce shell prouve composition, navigation,
état global, propagation de style et invalidation du rendu dans une application
réelle, sans lancer la réécriture du catalogue.

- **Button** : activation souris/clavier/tactile, focus visible, état désactivé
  ou occupé, variantes et sémantique de bouton. Vérifier qu'une mise à jour
  d'état ne reconstruit pas les sous-arbres voisins.
- **Select** : valeur contrôlée, options, ouverture et fermeture du popup,
  ancrage, navigation clavier, sélection, restauration du focus, action tactile
  et rôle/états accessibles. Réutiliser les mécanismes Rust génériques utiles
  sans conserver l'implémentation `.argui`.
- **Animation Lab** : reprendre les scénarios représentatifs de la page
  actuelle — opacité, taille/layout, rayon, rotation, couleur, timeline,
  keyframes, pause et retargeting. Les animations restent exécutées par le
  moteur natif ; un signal Solid ne doit pas traverser le pont à chaque frame.
  Mesurer aussi le CPU au repos lorsque tout est en pause.

Les autres widgets viendront **après** la validation de ce socle, au rythme
des besoins des applications. Leur absence ne bloque ni le choix du runtime,
ni l'architecture partagée Solid/React. Ne garder aucun placeholder `.argui`
pour ceux qui ne sont pas encore réimplémentés.

Sources : [Solid Universal](https://github.com/solidjs/solid/blob/main/packages/solid/universal/README.md),
[React reconciler](https://github.com/react/react/blob/main/packages/react-reconciler/README.md),
[architecture GPUIX](https://github.com/remorses/gpuix#architecture).

## Runtime JS : choix initial et possibilité de changer

Il faut séparer **outil de build**, **moteur exécutant l'application**, et
**pont vers Rust**. Bun peut compiler le TSX même si l'application finale tourne
sur un autre moteur ; le bundle applicatif ne doit pas appeler `Bun.*`, `Deno.*`
ou `node:*` dans les widgets et l'adaptateur commun. Les services fichier,
réseau, presse-papiers et notifications passent par des capacités Argui
explicites. Tester le bundle réel avec chaque moteur choisi.

| Candidat | Usage proposé | Point à vérifier |
| --- | --- | --- |
| **Bun** | Premier chemin pour obtenir rapidement un prototype **desktop** ; outil de build possible. `bun build --compile` inclut Bun, l'application et peut inclure l'addon Rust `.node` dans un exécutable. | Taille/RSS, boucle UI, callbacks N-API, paquetage macOS/Windows/Linux. Aucun engagement mobile fondé sur Bun. |
| **`deno_core`** | Candidat **desktop au même niveau de décision que Bun** : V8 intégré comme crate Rust, avec opérations Rust directes, sans addon N-API. | Compilation/packaging V8, intégration de la boucle UI et de l'event loop, taille/RSS, chargement d'un bundle JS précompilé. `deno_core` n'apporte pas toutes les API ni la résolution de modules du CLI Deno. |
| **QuickJS** | Second hôte embarqué à prototyper tôt, notamment pour Android/iOS et pour comparer mémoire/démarrage sur desktop. | Compatibilité du bundle Solid puis React, microtâches/timers, modules, absence d'`Intl`, débit JS et coût des appels natifs. Son API C ne charge pas directement un addon N-API. |
| **CLI Deno / `deno compile`** | Variante de distribution desktop à étudier seulement si le CLI complet apporte une capacité utile. | `deno compile` crée un exécutable à partir de `denort` ; cela n'embarque pas le CLI Deno comme bibliothèque dans le processus Rust. Le chargement d'addons natifs et le paquet final doivent être prouvés. `deno desktop` gère aussi sa propre fenêtre ; ne pas remplacer implicitement la fenêtre et le renderer Argui. |

**Objectif explicite : sélectionner le runtime au build**, par exemple avec
`argui build --js-runtime bun|deno-core|quickjs`. Chaque choix produit son
propre artefact avec le moteur retenu ; aucune permutation à chaud n'est prévue.
La sélection doit se limiter à un profil de build et à un bootstrap propre au
moteur, sans dupliquer l'application, l'adaptateur Solid/React ou `argui-host`.
On partage les mêmes sources TSX, le protocole de mutations et les tests ;
chaque hôte a sa boucle de tâches et son packaging. La cible est de pouvoir
compiler **le même benchmark** pour plusieurs moteurs avec une commande par
profil, puis comparer les résultats sur le même matériel. Si cela impose des
changements dans les widgets ou dans `UiTree`, l'interface commune est à revoir.

Faire le même petit écran interactif avec **Bun et `deno_core`**, puis retenir
le meilleur hôte desktop
mesuré ; prototyper QuickJS sur mobile avant d'ajouter une couche générique
plus large. Le `--engine quickjs` récent de Deno permet aussi une comparaison
desktop, mais reste expérimental et ne résout pas le packaging mobile. Si
QuickJS ne tient pas les budgets mobiles, évaluer Hermes ou JavaScriptCore
avec les mêmes tests plutôt que d'ajouter des abstractions sans usage.

Sources : [Bun standalone et addons](https://bun.com/docs/bundler/executables),
[Deno Core et ses opérations Rust](https://docs.rs/crate/deno_core/latest/source/README.md),
[Deno compile](https://docs.deno.com/runtime/reference/cli/compile/),
[Deno desktop et son backend brut](https://docs.deno.com/runtime/desktop/backends/),
[Deno Node-API](https://docs.deno.com/runtime/fundamentals/node/),
[QuickJS](https://bellard.org/quickjs/quickjs.html).

## Accessibilité et mobile : critères de livraison

`aria-*` est le vocabulaire du web. Le contrat Argui doit exposer des propriétés
sémantiques **typées** : rôle, nom/description, valeur et bornes, état
(`disabled`, `checked`, `expanded`, `invalid`, etc.), actions, relations
`labelledBy`/`describedBy`/`controls`, annonces et ordre de focus. Le renderer
les traduit vers l'arbre sémantique existant ; l'adaptateur web peut alors
produire ARIA, et les adaptateurs natifs utilisent les API de leur OS. Un
`Rectangle` décoratif ne devient pas automatiquement un contrôle accessible.

Dans la première tranche, tester clavier et lecteur d'écran sur **Button,
Select et la navigation du sidebar** : activation, ouverture, navigation des
options, annonce de la valeur et de la page courante, fermeture et restauration
du focus. Vérifier que les actions du lecteur d'écran
appellent le même comportement que le pointeur ou le clavier, et que les mises
à jour sémantiques seules ne relancent ni layout ni peinture. Les critères
équivalents pour Input, Dialog, Slider, listes virtualisées et formulaires
s'appliqueront lorsqu'ils seront implémentés, pas comme prérequis à ce socle.

Le risque mobile doit être résolu **avant** le port complet des widgets :

- L'adaptateur Android d'`accesskit_winit` 0.33.2 référence explicitement
  `GameActivity$InputEnabledSurfaceView`, alors qu'Argui démarre actuellement avec
  `NativeActivity`. Faire un prototype TalkBack : migration d'activité ou pont
  Java `AccessibilityNodeProvider` depuis le même arbre sémantique. Ne pas
  annoncer Android accessible avant un test réel.
- Sur iOS, conserver le rendu Rust/winit/UIKit. Tester l'adaptateur AccessKit
  iOS et VoiceOver sur simulateur **et** appareil ; les services Swift/SwiftUI
  peuvent rester natifs sans devenir le renderer des primitives Argui.
- Tester démarrage, pause/reprise, clavier logiciel, IME, safe areas, rotation,
  gestes, mémoire et destruction du runtime JS sur Android et iOS. Faire
  exécuter au moins un écran Solid puis un écran React sur chaque plateforme.

Sources : [AccessKit et ses adaptateurs](https://github.com/AccessKit/accesskit),
[compatibilité Android `accesskit_winit`](https://docs.rs/crate/accesskit_winit/latest),
[hiérarchie virtuelle Android](https://developer.android.com/reference/android/view/accessibility/AccessibilityNodeProvider),
[conteneurs accessibles iOS](https://developer.apple.com/documentation/uikit/uiaccessibilitycontainer).

## Étapes et preuves de sortie

| Étape | Travail | Preuve avant de passer à la suivante |
| --- | --- | --- |
| **A. Assainir** | Geler P4, inventorier les apports génériques du DSL, déplacer seulement ceux qu'utilise le cœur, retirer en une passe les 11 crates DSL, `.argui`, exemples, références Cargo/CI/scripts/docs et outils associés. Ne garder aucun shim. | Workspace et exemples restants compilent ; recherche de fichiers `.argui`, imports et commandes DSL active : zéro. État Git utilisateur préservé. |
| **B. Hôte natif** | Définir transactions, validation atomique, identités, callbacks et chemins modifiés. Mettre l'hôte derrière `argui-runtime`, sans dépendance JS dans `argui-ui`/`argui-render`. | Tests Rust des créations, mutations, reparenting, erreurs, démontage, focus/scroll/IME et classes d'invalidation. |
| **C. Solid vertical** | Adaptateur Universal, types générés, premier hôte desktop Bun, primitives nécessaires, **Button** Solid, topbar/sidebar et infrastructure de thèmes. | App native navigable et thème modifiable sans remonter l'arbre entier ; tests headless d'activation, d'états, d'événements et d'identité ; capture de rendu affecté sur l'affichage privé. Aucun DOM/WebView. |
| **D. Select et Animation Lab** | Ajouter **Select** Solid et sa sémantique, puis recréer **Animation Lab** en TSX ; les intégrer au sidebar ; récupérer uniquement les styles, assets et comportements Rust utiles à ces cas. Reporter les autres widgets. | Navigation entre les trois pages, Button et Select au clavier, clair/sombre et accents sur chacune ; Animation Lab valide retargeting, keyframes, invalidations et CPU au repos. |
| **E. Runtime et mobile tôt** | Porter le même shell et les trois cas sur `deno_core` et QuickJS ; exposer les profils de build ; prototyper le lancement Android/iOS. Choisir l'hôte desktop entre Bun et `deno_core`, et le profil mobile, sur données. | Une commande par runtime produit les artefacts du même benchmark, sans modification de l'application ; taille/RSS/latence mesurés ; sidebar, thème, Button et Select testés au tactile et au lecteur d'écran sur les plateformes annoncées. Si React ne passe pas sur le moteur mobile choisi, résoudre avant de promettre React mobile. |
| **F. React** | Adaptateur reconciler + shell, Button, Select et Animation Lab React sur la même API native. | Les fixtures communes donnent mêmes éléments, mutations, identité, focus, layout, animations, thème et sémantique que Solid ; test de rendu interactif. Aucun autre widget requis à ce stade. |
| **G. Livraison** | Retirer les chemins Rust visuels devenus inutiles, documenter packaging, choix du runtime, limites et matrice plateforme ; nettoyer toute promesse DSL. | Tests ciblés et mesures validés ; puis gate du dépôt une seule fois, immédiatement avant le commit final. |

### Mesures qui décident si le renderer est bon

Comparer en **release sur le même matériel**, avec scénario et nombre de nœuds
fixés : galerie Rust actuelle comme référence, Solid sur Bun, Solid sur
`deno_core` et QuickJS, puis React sur le runtime retenu. Enregistrer taille
distribuée, démarrage à froid, RSS/heap au repos et après montage, pic mémoire,
allocations, temps de sérialisation et d'application du batch, visites de
`UiTree`, coûts layout/texte/peinture, latence entrée→présentation p50/p95,
CPU idle et durée
de l'unmount/reload. Pour la première décision, inclure 10 000 primitives
montées, une propriété isolée, réordonnancement, changement de page et de thème,
Button activé, Select ouvert et fermé, et les scénarios de l'Animation Lab.
Reporter les benchmarks d'édition
de texte et de liste virtualisée au moment de ces fonctionnalités. Aucun frame
ne doit être planifié à l'idle ; les événements isolés ne doivent pas
reconstruire tous les descendants ni régénérer tout le display list.

Ce tableau est un **protocole de mesure**, pas une promesse de parité CPU/RAM
avec le Rust pur. GPUIX a mesuré que le modèle de données et la duplication des
styles pouvaient coûter davantage que le choix JSON/MessagePack : profiler
avant de changer de codec ou de remplacer `UiTree`.
[Mesures GPUIX](https://github.com/remorses/gpuix/blob/main/docs/serialization-benchmark.md).

### Ordre de grandeur et décision de runtime

Pour budgéter le chantier, compter environ **3 000 à 6 000 lignes** pour l'hôte
Rust, le protocole, les types générés, le pont Bun et l'adaptateur Solid ; puis
**800 à 1 800 lignes** pour l'adaptateur React et **700 à 1 500 lignes** pour
un hôte QuickJS réellement utilisable. Le prototype `deno_core` ajoute environ
**400 à 1 000 lignes** ; son éventuelle industrialisation dépendra de la
boucle d'événements et du packaging mesurés. La migration des widgets est un poste
distinct : environ **1 000 à 3 000 lignes** pour Button, Select et l'Animation
Lab, à préciser lors du port. Le reste du catalogue sera estimé séparément.
Ce sont des
estimations de code de production, hors tests et documentation, à réviser après
le premier `Rectangle` interactif. En contrepartie, les 11 crates DSL
représentent actuellement environ **55 000 lignes** de Rust et `.argui` dans
la branche auditée ; il faut dédupliquer et supprimer, pas transposer ce volume.

Ne pas annoncer un chiffre de RAM ou de débit « Solid » avant prototype : le
runtime, le graphe JS, les styles et les batches y contribuent séparément.
Retenir Bun ou `deno_core` sur desktop selon les budgets de démarrage, RSS et
latence définis avec la galerie Rust de référence ; choisir le moteur mobile
après mesure sur appareils. Le tableau de mesures ci-dessus devient un
rapport reproductible avec matériel, build, version du moteur et intervalles
observés. Si QuickJS est trop lent ou incompatible avec React, essayer Hermes
ou JavaScriptCore sur le **même contrat**, sans bifurquer le moteur Argui.

Pendant l'implémentation, tester seulement les crates et scénarios touchés.
Pour Linux GUI, utiliser `scripts/linux-hidden-display.sh` et automatiser
actions/assertions ; inspecter une capture seulement quand peinture/layout
changent. Après implémentation complète, appliquer une seule fois le gate décrit
dans `docs/contributing/code-quality.md`. Ce plan n'exécute aucun build ni test
et ne valide aucune plateforme à lui seul.
