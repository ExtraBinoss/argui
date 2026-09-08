# Observations sur la maturité d’Argui

État de l’audit : 6 septembre 2026, sur l’arbre de travail incluant les changements
non commités. Ce document conserve les observations pour de futurs chantiers ;
il ne constitue ni une promesse de compatibilité, ni un ordre de lancement de
leur implémentation. Revalider les constats avant chaque chantier.

## Synthèse

Argui possède déjà une vraie base de framework GUI. L’écart avec un framework
comme GPUI concerne surtout la maturité de l’API applicative, l’extensibilité et
les garanties de livraison, pas seulement le nombre de widgets ou d’effets.

Les fondations existantes incluent le rendu WGPU natif/Web, le layout retenu,
le texte mis en forme, les entités et leur invalidation, la virtualisation,
les animations et effets, les tâches annulables, les actions selon le focus,
undo/redo, Password, les sémantiques d’accessibilité et la WebView optionnelle.
La priorité proposée est de consolider ces fondations plutôt que de réécrire
le moteur ou de multiplier les effets graphiques.

## 1. État partagé et cycle de vie

**Réévaluation du 7 septembre 2026 :** les observations initiales ci-dessous ne
décrivent plus toutes le code actuel. Modèles sans rendu, abonnements publics et
montages indépendants existent et sont testés. La clôture du point 1 suit désormais
[les preuves et exigences P1-01 à P1-08](point-1-completion.md), avec les limites
de couverture et les scénarios manuels. Les paragraphes suivants conservent le
constat initial, pas une liste actuelle de fonctionnalités absentes.

**Présent :** `Entity`, `WeakEntity`, cache de rendu, invalidation et tâches
appartenant à une entité.

**Partiel :** les opérations sur les entités et leur contexte restent liées à
`Render`. L’observation interne du cache ne constitue pas une API publique
d’abonnements entre modèles applicatifs.

**À étudier :**

- Modèles de données indépendants du rendu.
- Émission et abonnement à des événements typés entre modèles.
- Abonnements détachables, avec libération et ownership explicites.
- Cycle de vie clair pour montage, démontage, rétention et destruction.
- État et services partagés entre fenêtres, avec ownership des tâches explicite.

Exemple de validation : une boîte mail, un compteur de messages non lus et deux
fenêtres observent le même modèle ; fermer une vue libère ses abonnements sans
détruire le modèle encore utilisé ailleurs.

Sources : [modèle runtime](../crates/argui-runtime/src/model.rs),
[cache](../crates/argui-runtime/src/model/cache.rs), [contrat des tâches](tasks.md).

## 2. Éléments personnalisés

**Présent :** composition des primitives, vecteurs, shaders et effets personnalisés.

**Limite :** `ElementKind` est une liste fermée de primitives. Je ne trouve pas de
contrat public équivalent à un élément définissant lui-même son layout, ses
zones interactives et son rendu sans modification du moteur.

**À étudier :** un contrat d’extension couvrant mesure/layout, préparation du
rendu, peinture, interaction, accessibilité et invalidation. Il doit préserver
les chemins incrémentaux et rester indépendant d’un éventuel DSL.

Exemple de validation : une timeline ou un graphe interactif dans une crate
consommatrice, sans ajouter de variante spécifique dans le cœur d’Argui.

Source : [ElementKind](../crates/argui-ui/src/element/kind.rs).

## 3. Intégration système

**Présent :** fenêtres, presse-papiers, tray et WebView optionnelle.

**À compléter :**

- Dialogues natifs d’ouverture et d’enregistrement de fichiers.
- Menus natifs raccordés aux actions applicatives.
- Drag-and-drop interne et système, avec données et opérations explicites.
- Contrats de capacités, annulation et erreurs selon la plateforme.

Exemple de validation : déposer une pièce jointe depuis le bureau, déplacer un
message entre dossiers et enregistrer une pièce jointe via un dialogue système.
Le Web doit annoncer ses limites, pas simuler une capacité indisponible.

Sources : [événements plateforme](../crates/argui-platform/src/event.rs),
[commandes applicatives](../crates/argui-runtime/src/application.rs).

## 4. Actions et clavier

**Présent :** actions partagées entre boutons, menus, palette et raccourcis ;
disponibilité et résolution selon le focus.

**À compléter :** raccourcis configurables, séquences de touches, actions
paramétrées et diagnostics de conflits. Ne pas remplacer le mécanisme existant
sans vérifier ce qui peut être étendu proprement.

Exemple de validation : remapper « archiver », définir une séquence de touches
et invoquer « déplacer vers ce dossier » depuis un menu ou une palette, avec
les mêmes règles de disponibilité.

Source : [actions](../crates/argui-ui/src/action.rs).

## 5. Interfaces de données

**Mise à jour du 8 septembre 2026 :** `List`, `VList` à hauteurs fixes ou
mesurées et `Table` sont exposés comme widgets réutilisables. Ils partagent une
sélection contrôlée simple/multiple, Shift/Ctrl/Cmd, flèches, Home/End et des
sémantiques accessibles. Trois pages sobres remplacent les démonstrations
applicatives de listes dans la galerie.

Les mesures variables persistent entre rendus et redimensionnements. Le moteur
corrige l'ancrage lors des mesures ; insertion/suppression et tri exigent encore
un remappage explicite de la sélection et de l'offset par le propriétaire.
La table de base monte toutes ses lignes. Virtualisation de table, tri/filtres,
pagination et navigation par cellule restent à développer. La validation avec
lecteurs d'écran réels n'est pas remplacée par les tests sémantiques.

Sources : [API et limites](lists-tables.md), [checklist shadcn](shadcn-lib.md),
[virtualisation](../crates/argui-ui/src/virtual_list.rs),
[List](../crates/argui-widgets/src/list.rs),
[VList](../crates/argui-widgets/src/vlist.rs),
[Table](../crates/argui-widgets/src/table.rs).

## 6. Texte et édition

**Présent :** champs éditables, texte mis en forme, IME, undo/redo transactionnel
borné et champs Password protégés.

**Limite :** ce n’est pas encore un éditeur de documents riches. L’affichage de
texte enrichi n’équivaut pas à un modèle de document éditable avec structure et
opérations de mise en forme. Ne pas confondre les capacités du framework GPUI
avec celles de l’éditeur Zed construit dessus.

À approfondir selon les besoins : composition de messages riches et validation
réelle des interactions IME, bidi, sélection et historique sur chaque plateforme.

Sources : [guide actions/édition](actions-editing.md), [saisie texte](text_input.md).

## 7. API de test pour les applications

**Présent :** nombreux tests internes et quelques scénarios navigateur.

**À ajouter :** un harnais public de type `TestApp` permettant de piloter les
clics, le clavier, le focus, les fenêtres, une horloge et les tâches, sans que
chaque application reconstruise son propre environnement de test.

Exemple de validation : ouvrir une palette au clavier, lancer une recherche,
remplacer cette recherche, livrer les résultats dans le désordre et vérifier
le focus et le résultat affiché de manière déterministe.

## 8. Validation visuelle, accessibilité et performances

**Présent :** profiling, optimisations retenues, tests logiques et outils de
mesure. L’accessibilité a déjà ses adaptateurs et son arbre sémantique.

**À renforcer :**

- Scénarios visuels automatisés pour effets, clips et overlays.
- Audits avec lecteurs d’écran et IME réels.
- Validation Linux, Windows, macOS et Web, au-delà de la compilation.
- Budgets mesurés de latence, mémoire, allocations et travail par frame.
- Mesures comparables avec/sans DevTools, pendant scroll, resize et animations.

Ne pas revendiquer des performances « au niveau de GPUI » sans benchmark
comparable. Un taux de couverture élevé ne prouve ni la fluidité ni la qualité
visuelle ou l’accessibilité d’une application.

Sources : [accessibilité](accessibility.md), [architecture](architecture.md),
[roadmap existante](roadmap.md).

## 9. Distribution et contrat de bibliothèque

**Présent :** workspace expérimental, features opt-in et documentation technique.

**À préparer :** publication des crates et de leurs dépendances internes,
politique de compatibilité, fichiers de licence du projet, changelog, CI
multiplateforme et validation des combinaisons de features.

L’audit n’a pas trouvé de workflows sous `.github/workflows`, ni de fichiers
de licence du projet à la racine. Les licences de ressources tierces présentes
ne remplacent pas ces fichiers. Les dépendances internes sont déclarées par
chemin dans le workspace.

Exemple de validation : créer une application dans un dépôt distinct, avec les
seules features nécessaires, et la compiler selon la matrice officiellement
supportée en suivant la documentation publique.

Sources : [workspace](../Cargo.toml), [façade Argui](../crates/argui/Cargo.toml).

## Ordre proposé

1. État partagé, abonnements et cycle de vie.
2. API de test applicative et validation multiplateforme.
3. Services système et drag-and-drop, notamment pour le client mail.
4. Éléments personnalisés et composants de données avancés.

Préparer la distribution et stabiliser les contrats au fil de ces étapes.
Les extensions de l’édition et des raccourcis doivent être guidées par des
besoins applicatifs concrets, pas par une recherche de parité exhaustive.

## Références GPUI consultées lors de l’audit

GPUI sert de référence architecturale, pas de spécification à copier ni de
garantie universelle de stabilité. Les pages ci-dessous peuvent évoluer.

- [Vue d’ensemble et tests applicatifs](https://docs.rs/gpui/latest/gpui/).
- [Context : observation, abonnements et cycle de vie](https://docs.rs/gpui/latest/gpui/struct.Context.html).
- [Element : layout, préparation et peinture](https://docs.rs/gpui/latest/gpui/trait.Element.html).
- [App : services système](https://docs.rs/gpui/latest/gpui/struct.App.html).

## État des vérifications à la date de l’audit

- 771 tests passent ; aucun test ignoré dans cette exécution.
- Clippy natif et compilation Web passent après correction du ciblage d’un
  test de `spawn_blocking`, réservé au natif.
- Couverture affichée par le script, tronquée à l’entier : branches 84 %,
  fonctions 88 %, lignes 89 %, régions 89 %.
- Le contrôle de couverture exige 90 % sur chaque métrique et échoue.
- Aucun commit/push effectué à l’issue de cet audit ; aucune dérogation implicite.

Ces chiffres décrivent cette exécution, pas une garantie pour les changements
futurs. Le présent document ne déclenche pas une nouvelle campagne de tests.
