# Argui DSL — Checklist de consolidation et d’UI générale

Plan de référence : [GENERAL_UI_PLAN.md](GENERAL_UI_PLAN.md).
Date : 23 septembre 2026. État : checkpoint de sauvegarde sur `codex/dsl-gallery-live` ; développement DSL arrêté à la demande de l’utilisateur.

## Mode d’emploi

Suivre les phases et dépendances du plan. Cocher uniquement après implémentation
et vérification du comportement. La présence d’une API, d’un nom dans le schéma
ou d’une page de galerie ne suffit pas.

À la fin de chaque phase, remplir sa ligne dans le registre de preuves : fichiers,
tests/commandes exécutés, résultats, captures applicables et limites restantes.
Un test déjà présent mais non exécuté doit être identifié comme tel. Une phase
bloquée reste ouverte avec une raison précise et la prochaine action possible.

## Préparation

- [x] Réaliser l’audit initial code/tests et les sondes CLI décrites dans le plan.
- [x] Écrire le plan avec responsabilités, dépendances et critères de sortie.
- [x] Écrire cette checklist sans annoncer de capacité nouvellement implémentée.
- [x] Commencer l’implémentation après la demande de l’utilisateur.

## P0 — Baseline et reproductions

- [x] Lire les règles de qualité et de tests GUI ; relever commit et état Git.
- [ ] Inventorier capacités implémentées, tests existants et preuves manquantes.
- [ ] Reproduire l’affectation de type invalide dans un handler.
- [x] Reproduire l’écriture externe dans une propriété privée/de sortie.
- [ ] Reproduire le repeater sans clé et vérifier son arbre effectivement produit.
- [x] Reproduire la divergence de court-circuit AOT/live.
- [ ] Reproduire la fourniture de plusieurs slots et l’application d’un style.
- [x] Reproduire le retrait d’un enfant monté pendant un reload.
- [ ] Enregistrer les résultats et les tests ciblés de départ.

## P1 — Garanties du langage

- [ ] Valider destinations, types et opérateurs des affectations.
- [ ] Respecter les directions `in`, `out`, `in-out` et `private`.
- [ ] Valider retours de handlers et liaisons bidirectionnelles.
- [ ] Rejeter les repeaters sans clé avec un diagnostic source.
- [ ] Définir et vérifier portée, types et collisions des clés.
- [ ] Garantir le court-circuit identique de `&&`, `||` et des conditionnelles.
- [ ] Séparer fonctions pures et callbacks à effets dans les bindings/handlers.
- [ ] Spécifier et vérifier conversions, unités, optionnels et erreurs arithmétiques.
- [ ] Afficher les warnings via CLI/LSP sans les perdre après compilation.
- [ ] Compiler réellement les fixtures AOT et exécuter les mêmes fixtures en live.
- [ ] Vérifier diagnostics des cas invalides et parité des résultats valides.
- [ ] Mettre à jour le contrat de langage et enregistrer les preuves P1.

## P2 — Composition et styles

- [ ] Définir et implémenter la fourniture explicite de slots nommés.
- [ ] Gérer slots requis/optionnels, cardinalité et contenu par défaut.
- [ ] Permettre le forwarding des slots à travers des composants imbriqués.
- [ ] Ajouter des paramètres typés aux templates de contenu.
- [ ] Préserver la construction paresseuse des lignes virtualisées.
- [ ] Définir les références lexicales par instance dans les repeaters.
- [ ] Gérer explicitement l’absence d’un enfant conditionnel référencé.
- [ ] Appliquer réellement les styles nommés en AOT et live.
- [ ] Tester précédence des défauts, thèmes, styles, inline, états et animations.
- [ ] Vérifier portée et surcharge des tokens de thème.
- [ ] Prouver Dialog header/body/actions et champ leading/trailing/error.
- [ ] Documenter syntaxe et erreurs, puis enregistrer les preuves P2.

## P3 — Layout et propriétés visuelles

- [ ] Exposer Grid, tracks, fractions, min/max, placement et spans.
- [ ] Exposer max-size, aspect ratio, flex basis et alignements nécessaires.
- [ ] Exposer espacements par côté avec des types/unités cohérents.
- [ ] Exposer les règles responsive basées sur le conteneur.
- [ ] Définir les lectures de dimensions mesurées et prévenir les boucles de layout.
- [ ] Définir positionnement, superposition et clipping composables.
- [ ] Exposer transformations composées, origine, translation et échelle.
- [ ] Exposer coins indépendants, bordures par côté et ombres des preuves.
- [ ] Vérifier cohérence peinture/hit testing sous transformations et clipping.
- [ ] Prouver dashboard et formulaire à plusieurs tailles/échelles d’affichage.
- [ ] Inspecter captures privées et enregistrer les preuves P3.

## P4 — Expressions et modèles

- [ ] Ajouter l’indexation sûre et définir le résultat hors bornes.
- [ ] Ajouter les littéraux de structs et valeurs d’enums.
- [ ] Compléter l’utilisation explicite des optionnels.
- [ ] Implémenter les variables locales et branches dans les handlers.
- [ ] Implémenter les fonctions pures typées et le suivi de leurs dépendances.
- [ ] Distinguer contrats `array<T>` et `model<T>` dans les deux backends.
- [ ] Définir identité, lecture et mutations insert/remove/move/update-row.
- [ ] Définir ownership, abonnements, destruction et migration des modèles.
- [ ] Fournir projections tri/filtre nécessaires à la table de preuve.
- [ ] Conserver la virtualisation et ajouter les hauteurs variables mesurées.
- [ ] Préserver focus, sélection et ancrage de scroll pendant les mutations.
- [ ] Mesurer lignes montées, copies et évaluations sur une mise à jour isolée.
- [ ] Enregistrer les preuves P4 et les limites de coût documentées.

## P5 — API Rust, services et extensions

- [ ] Générer une façade publique typée commune AOT/live.
- [ ] Couvrir propriétés, callbacks, types utilisateur, modèles et slots.
- [ ] Retirer les bridges applicatifs dépendant de l’ordre des callbacks ou de l’IR.
- [ ] Brancher loading/result/error sur les tâches du runtime.
- [ ] Gérer annulation, dernière requête gagnante et destruction du composant.
- [ ] Rendre le file picker et la recherche non bloquants pour l’UI.
- [ ] Injecter le même registre de primitives dans compilation, AOT, live et tooling.
- [ ] Définir identité/version/ABI et diagnostics des extensions natives.
- [ ] Tester les propriétés, callbacks, slots et assets d’une primitive externe.
- [ ] Prouver une extension dans une crate applicative sans modification d’Argui.
- [ ] Documenter la frontière rebuild Rust/reload DSL et enregistrer les preuves P5.

## P6 — Dessin réactif et gestes

- [ ] Définir commandes vectorielles dynamiques, fill/stroke et validation.
- [ ] Relier points et tracés aux propriétés et modèles réactifs.
- [ ] Préserver l’optimisation des assets vectoriels statiques.
- [ ] Définir identité, cache, invalidation et libération des géométries.
- [ ] Exposer pan/pinch/rotation avec phases, coordonnées et annulation.
- [ ] Définir capture, propagation et arbitrage avec le scroll.
- [ ] Vérifier les interactions après translation/zoom/rotation.
- [ ] Prouver déplacement de nœuds et connexions sans recompilation d’assets.
- [ ] Mesurer invalidation et ressources ; enregistrer les preuves P6.

## P7 — Accessibilité, texte et internationalisation

- [ ] Exposer valeurs/bornes/actions accessibles et états pertinents.
- [ ] Exposer relations label, description, controls et active-descendant.
- [ ] Exposer annonces et métadonnées de listes/grilles.
- [ ] Compléter Slider : flèches, Home/End, valeur et actions accessibles.
- [ ] Vérifier menus, tabs, sélection, combobox, dialogs et restauration du focus.
- [ ] Exposer familles et assets de polices.
- [ ] Ajouter les spans typés au sein d’un même flux de texte.
- [ ] Vérifier wrapping, sélection, IME et caret multilingues.
- [ ] Ajouter les arguments typés de traduction et les pluriels.
- [ ] Rendre le changement de locale réactif et la direction RTL/LTR héritée.
- [ ] Vérifier propriétés logiques et positionnement des overlays en RTL.
- [ ] Prouver le formulaire français/arabe au clavier et sémantiquement.
- [ ] Enregistrer les preuves P7, dont les captures privées applicables.

## P8 — Live et outils

- [ ] Démonter les instances dont la définition disparaît lors d’un reload.
- [ ] Tester insertion, retrait, remplacement, réordonnancement et reparenting.
- [ ] Définir les garanties d’identité automatique, `#id` et clés.
- [ ] Vérifier conservation ou reset intentionnel de l’état par cas.
- [ ] Vérifier focus, sélection, scroll, overlays et animations pendant le reload.
- [ ] Vérifier l’ABI des types publics transitifs et des extensions natives.
- [ ] Conserver la dernière génération valide après rejet et reprendre après correction.
- [ ] Vérifier la mise à jour visible sans clic ou entrée supplémentaire.
- [ ] Synchroniser LSP, changements disque et diagnostics des modules dépendants.
- [ ] Relier élément inspecté, composant, source et bindings.
- [ ] Exposer erreurs de reload et causes d’invalidation dans les outils existants.
- [ ] Enregistrer les preuves P8, dont ressources après reloads répétés.

## P9 — Applications de preuve

- [ ] Table : chargement asynchrone, erreurs, annulation et modèles mutables.
- [ ] Table : tri/filtre, cellules custom, édition, sélection et validation.
- [ ] Table : dialogue à plusieurs slots et liste virtualisée.
- [ ] Éditeur de nœuds : déplacement, sélection et raccourcis.
- [ ] Éditeur de nœuds : connexions dynamiques, zoom et surface externe.
- [ ] Formulaire : français/arabe, pluriels et texte riche.
- [ ] Formulaire : responsive, erreurs accessibles et clavier complet.
- [ ] Exécuter les trois preuves en AOT et live, avec reload en cours d’interaction.
- [ ] Relier l’inventaire galerie à sa matrice de capacités et validations.
- [ ] Identifier les pages partielles et les services hôtes réellement branchés.
- [ ] Comparer layout, focus, sélection, offsets, rendu et sémantique.
- [ ] Inspecter les captures non blanches sur l’affichage Linux privé.
- [ ] Mesurer idle, mutations, virtualisation et ressources ; enregistrer P9.

## P10 — Livraison

- [ ] Mettre à jour guides, inventaires et anciennes checklists.
- [ ] Retirer les API remplacées, leurs appelants et les promesses non implémentées.
- [ ] Vérifier Rustdoc, séparation des responsabilités et limite de 600 lignes Rust.
- [ ] Exécuter les tests ciblés avec `cargo nextest run --all-features`.
- [ ] Vérifier les exemples dans leur workspace et les API publiques exécutables.
- [ ] Vérifier release, assets embarqués et graphe de dépendances AOT.
- [ ] Consigner séparément compilation et exécution web/mobile/native.
- [ ] Relire le diff et préserver les modifications utilisateur.
- [ ] Après implémentation, immédiatement avant commit : exécuter exactement une
  fois `./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh`.
- [ ] Confirmer 85 % minimum par métrique et par crate, ainsi que pour le total.
- [ ] Consigner tout échec/blocage ; ne cocher aucune vérification non obtenue.
- [ ] Enregistrer les preuves finales et ne déclarer terminées que les phases validées.

## Registre des preuves

Les cases restent ouvertes lorsqu’une preuve de sortie complète manque. Voir
[GENERAL_UI_HANDOFF.md](GENERAL_UI_HANDOFF.md) pour les résultats ciblés de ce
checkpoint ; la fusion `codex/gallery-text-fidelity` est effectuée.

| Phase | État | Fichiers/tests et résultats | Captures/mesures | Limites ou prochaine action |
| --- | --- | --- | --- | --- |
| P0 | Baseline établie | [Baseline](GENERAL_UI_BASELINE.md) | — | Ne pas refaire l’audit. |
| P1 | Correctifs principaux présents | 18 scénarios AOT/live déjà passés | — | Revenir seulement pour défaut concret. |
| P2 | Presque complet, preuves ciblées | Slots, templates, refs, styles ; sélections ciblées vertes | — | Portée des tokens et limites à documenter. |
| P3 | Partiel | Grid, responsive et dimensions mesurées AOT/live ; garde directe testée | Aucune capture finale | Peinture, hit testing et échelles à vérifier. |
| P4 | Partiel | Modèles, liste riche et fonctions pures AOT/live ; diagnostics ciblés verts | Aucun coût chiffré | Focus/sélection/ancrage et coûts à prouver. |
| P5 | Partiel | Façade live, `ProofTile`, services hôte ajoutés ; tests ciblés façade/extension verts | Service GUI non vérifié | Test service, ABI mutation et parité complète. |
| P6–P10 | Non commencés | — | — | Hors de cette conversation ; décider d’abord si le DSL est conservé. |

## Performance et vérifications ciblées

- Galerie DSL : les animations auparavant figées ou déclenchées seulement par
  l’utilisateur tournent maintenant en boucle. `playing` suspend les motions sans
  perdre leur phase. Le test ciblé vérifie les 14 pistes sur plusieurs cycles
  en AOT et live, ainsi que pause et reprise.
- Le cache des éléments natifs réutilise les sous-arbres inchangés. Les hauteurs
  du layout ne provoquent un nouveau rendu DSL que pour les listes virtualisées
  suivies. Les valeurs par défaut remplacées par un binding ne sont plus évaluées
  à chaque frame. Les références d’enfants utilisées sont calculées au chargement
  du package, plutôt que par parcours de l’IR à chaque rendu.
- Vérifications au 23 septembre : 18/18 scénarios de conformité AOT/live ;
  203/204 tests ciblés runtime/compilateur/protocole lors de la première passe.
  Le seul échec était une assertion de texte sur la forme du Rust généré, mise à
  jour puis validée avec le nouveau test du binding remplacé : 2/2 ciblés.
  Galerie : 25/26 avant correction du délai de fin de transition de navigation.
- La navigation de la galerie monte maintenant seulement les lignes visibles dans
  un `VirtualWindow` ; la molette à petites impulsions et le scrollbar ont été
  vérifiés sur l’affichage privé. Ses mesures CPU sont non concluantes car la
  surface renvoie ensuite `RenderStatus::Skipped`, y compris pour le spinner de
  la galerie Rust native. Une trace temporaire, retirée ensuite, a confirmé
  que les motions DSL avancent et que les premières frames sont présentées.
- L’utilisateur a testé sur son bureau la galerie AOT : ~0,8 % CPU sur Button,
  1,5 % maximum dans Animation Lab, 0 % quand aucune animation ne joue et
  ~1,1 % dans Damage Control. Toutes les animations fonctionnent. Avec
  `argui dev`, il constate seulement 0,1–0,2 point d’écart CPU. Ces mesures
  manuelles rendent la performance acceptable pour avancer ; elles ne sont
  pas un benchmark automatisé et précèdent la fusion de
  `codex/gallery-text-fidelity` dans `codex/dsl-gallery-live`, maintenant réalisée.
- Le benchmark headless mesure des temps et des ticks CPU dans un profil de test
  non optimisé ; des compilations concurrentes ont perturbé sa durée.
- `./scripts/quality.sh` et la couverture LLVM finale n’ont pas été exécutés.
  Ils doivent rester réservés à la fin réelle de P10, une seule fois avant commit.
