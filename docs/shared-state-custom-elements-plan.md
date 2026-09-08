# Plan — état partagé, cycle de vie et éléments personnalisés

## Objectif et périmètre

Résoudre entièrement les points 1 et 2 de [l’audit](oberservations.md).
Ce document définit le travail et ses critères d’acceptation ; il ne déclare
pas les fonctionnalités terminées. Le [suivi d’implémentation](shared-state-custom-elements.md)
reste distinct des engagements ci-dessous.

Le code actuel fournit déjà des entités de données sans `Render`, un dispatcher
de modèles, des événements typés, des abonnements et des portées de ressources.
Il reste notamment à séparer la présentation de l’entité, raccorder ces contrats
aux vrais hôtes et ouvrir le pipeline aux éléments personnalisés.

Pas de seconde architecture parallèle, d’adaptateur de compatibilité durable,
de capacité simulée ou de nouvelle dépendance sans nécessité démontrée.
Les noms d’API ci-dessous décrivent le contrat cible ; leur orthographe finale
sera arrêtée avec les tests consommateurs, avant migration des appelants.

## 1. Modèles, propriété et cycle de vie

La [checklist de clôture du point 1](point-1-completion.md) confronte les lots A–D
aux tests existants et précise les preuves manquantes, la couverture disponible
et les vérifications utilisateur. Elle ne remplace ni ne réduit les exigences
ci-dessous ; elle distingue les contrats déjà exercés de leur intégration à prouver.

### Lot A — séparer données et présentation

- Conserver `Entity<T>` et `WeakEntity<T>` pour tout modèle de données `'static`.
  Réserver `Render` aux vues ; aucun modèle métier ne doit devoir le simuler.
- Séparer contexte de modèle et contexte de vue. Le premier donne accès aux
  données, événements, abonnements, services et tâches ; le second ajoute
  fenêtre, focus, layout, événements UI, peinture et animations.
- Faire posséder le runtime et les services partagés par l’application, sans
  singleton caché. Les modèles restent sur le thread UI ; les workers échangent
  des messages transférables, pas des références aux objets UI.
- Distinguer identités d’entité, de fenêtre et de montage. Sortir de l’entité
  les caches de présentation, handlers, dépendances de rendu et environnements
  qui appartiennent à un montage. L’arbre UI garde son layout et son focus locaux.
- Monter une même vue dans deux fenêtres doit produire deux présentations
  indépendantes, même lorsque leur entité et leurs données sont partagées.

Validation : modèle sans `Render`, deux fenêtres avec tailles, DPI et thèmes
différents ; aucun handler, focus, rectangle ou cache ne passe d’une fenêtre
à l’autre. Modifier le modèle invalide les deux présentations concernées.

### Lot B — événements et observation publics

- Finaliser `EventEmitter<E>`, émission typée, observation des modifications et
  lecture suivie pendant le rendu. Documenter leurs différences : une mutation
  invalide des dépendances, un événement transporte une information métier.
- Livrer les callbacks après libération des emprunts du modèle, dans un ordre
  déterministe. Définir les règles de réentrance et de souscription pendant
  une livraison ; ne pas exécuter de callback sous un emprunt de registre.
- Garder les endpoints faibles et les abonnements RAII explicitement détenus.
  Annulation et destruction suppriment aussi les livraisons déjà en attente.
  Ne pas proposer de détachement sans propriétaire de remplacement explicite.
- Conserver bornes de file, erreurs de saturation et budget de traitement.
  Propager les invalidations de manière itérative, avec coalescence et protection
  contre les cycles ; aucune boucle métier ne monopolise le thread UI.
- Définir la frontière entre applications : pas d’abonnement direct implicite
  entre runtimes indépendants. Les fenêtres d’une application partagent le même
  domaine ; les messages externes passent par son point d’entrée explicite.
- Raccorder le dispatcher aux boucles natives et Web : réveils regroupés,
  reprise du travail restant, redraw des seules fenêtres sales, repos sans polling.

Validation : réentrance, destruction pendant livraison, annulation en file,
ordre et saturation, grand nombre d’observateurs, cycles, travail réparti sur
plusieurs tours de boucle et absence de résultat bloqué jusqu’au prochain clic.

### Lot C — propriété et tâches

- Formaliser les portées application, fenêtre, entité et montage avec fermeture
  idempotente. Un montage appartient à une fenêtre et retient son entité ; son
  démontage libère ses ressources sans détruire une entité retenue ailleurs.
- Distinguer création, montage, changement de visibilité, démontage et destruction.
  Masquer une vue ne signifie pas la démonter. Un remontage crée une nouvelle
  identité de montage ; l’état métier retenu survit, pas ses anciens handlers.
- Livrer les notifications de cycle de vie aux frontières de transaction, hors
  mesure/layout/paint. Documenter l’ordre parent/enfant et le traitement d’une
  mutation ou fermeture demandée depuis ces notifications.
- Réutiliser `ResourceScope` pour abonnements, tâches et ressources libérables.
  Retirer immédiatement les inscriptions terminées ou annulées ; les handles
  survivants ne doivent pas retenir involontairement leurs propriétaires.
- Donner à chaque tâche un propriétaire explicite. Fermer une vue annule ses
  tâches locales, jamais une synchronisation appartenant au modèle partagé.
  Fermer l’application arrête son executor et interdit de nouvelles livraisons.
- Protéger les résultats périmés par identité/génération et politique de
  remplacement. Préciser qu’un travail bloquant déjà lancé ne peut pas toujours
  être interrompu : son résultat devient inapplicable et la coopération est explicite.
- Supprimer la logique de cancellation fondée seulement sur le sous-arbre rendu
  et les anciens chemins d’attachement d’executor après migration des consommateurs.

Validation : fermeture d’une des deux fenêtres, destruction du dernier handle,
montage/démontage répété, ressource partiellement initialisée, nettoyage réentrant,
tâches terminées dans le désordre et fermeture pendant leur livraison.

### Lot D — exemple « Shared Mailbox » et migrations

- Ajouter une page de galerie : modèle de messages partagé, compteur non lus,
  liste virtualisée, détail et actions marquer lu/non lu.
- Ajouter une vraie deuxième fenêtre native observant le même modèle, avec
  sélection et scroll indépendants. Fermer puis rouvrir cette fenêtre.
- Montrer démontage/remontage d’une vue et tâche locale annulable, distincte
  d’une tâche du modèle. Utiliser un jeu de données local déclaré comme tel,
  sans simuler un serveur de messagerie ni prétendre mesurer un accès réseau.
- Sur Web, démontrer les vues partagées et le cycle de vie dans le navigateur ;
  ne pas appeler une seconde colonne « fenêtre native ». Présenter explicitement
  la disponibilité du scénario multi-fenêtre selon le backend.
- Migrer les hôtes mono/multi-fenêtres, GTK, Web, wrappers de sélection,
  DevTools et usages de la galerie. Supprimer les API remplacées.

## 2. Éléments personnalisés

### Lot E — contrat public et identité retenue

- Introduire une extension générique de `ElementKind`, pas une variante par
  timeline, graphe ou application. La galerie doit l’utiliser sans API privée.
- Exposer propriétés typées, état retenu par montage, révision explicite et
  contexts de phase. L’effacement de type reste interne, contrôlé à la frontière.
- Définir ce qui conserve l’état : identité stable et type compatible. Changement
  de type, retrait ou remplacement libèrent l’ancien état et ses ressources.
  Signaler les identités dupliquées au lieu de réutiliser silencieusement un état.
- Prévoir feuilles et conteneurs avec enfants Argui ordinaires. Une extension
  ne possède ni sa propre boucle d’événements ni son propre renderer WGPU.
- Répartir les contrats sans cycle de dépendances : UI pour description et
  interaction, layout pour mesure et placement, paint pour commandes graphiques,
  runtime pour montages et services. Les types de frontière vont dans le niveau
  neutre le plus bas qui les utilise réellement, sans dépendance à un DSL.

### Lot F — mesure, layout et préparation

- Donner des contraintes explicites, tailles intrinsèques et baseline ; permettre
  de mesurer les enfants puis de les placer via le moteur existant.
- Spécifier les contraintes finies/non bornées, minimum/maximum, taille nulle,
  clipping, scroll, transforms et conversion logique/physique selon le DPI.
- Rejeter les tailles négatives/non finies et détecter les cycles ou l’absence
  de convergence avec un diagnostic contextualisé. Aucun fallback silencieux
  ne doit cacher une extension invalide.
- Séparer préparation géométrique et peinture. Autoriser les caches de texte,
  géométrie et assets avec dépendances et invalidation explicites.
- Interdire mutation du modèle et effets de bord applicatifs pendant mesure et
  peinture ; faire remonter les demandes de mise à jour via les phases autorisées.

Validation : feuille et conteneur externes, contraintes adverses, baseline,
enfants standard, scroll imbriqué, clipping transformé et changement de DPI.

### Lot G — peinture et interaction

- Fournir un contexte renderer-neutral : texte mis en forme, vecteurs, images,
  clips, transforms, couches, opacité et effets déjà pris en charge par Argui.
  Les ressources suivent les contrats d’assets existants et sont libérées.
- Préserver ordre de peinture, z-order, overlays et isolation des clips ; ne pas
  rendre l’extension dans une texture offscreen si la composition ne le demande pas.
- Déclarer des sous-régions interactives à identités stables, avec hit-test,
  curseur, focus, capture et routage via le système d’événements existant.
- Faire fonctionner sur ces régions gestes, clavier et actions applicatives.
  Définir la priorité entre région personnalisée, enfants standard et overlays.
- Retrait d’une région : libérer capture, hover, handlers et focus selon la
  politique du moteur, y compris au milieu d’un drag ou d’un événement.

Validation : hit-test sous transforms/clips, capture hors limites, suppression
pendant drag, fermeture de fenêtre, interaction avec un bouton enfant et overlay.

### Lot H — accessibilité, invalidation et DevTools

- Exposer un sous-arbre sémantique avec identités, rôles, labels, valeurs, bornes
  et actions raccordées aux mêmes comportements que le pointeur/clavier.
- Associer régions interactives, nœuds sémantiques et focus. Les adaptateurs
  natif/Web doivent recevoir les mises à jour et suppressions, sans doublons.
- Distinguer invalidation de mesure/layout, préparation/paint, interaction et
  sémantique ; documenter quelles dépendances entraînent les autres phases.
- Préserver les caches quand seules la couleur, la sélection ou les sémantiques
  changent. Pas de reconstruction générale ni d’animation permanente par défaut.
- Exposer dans les DevTools type, identité, contraintes, bornes, sous-régions,
  sémantiques, causes d’invalidation et coûts des phases, sans fuite de données
  protégées ni instrumentation coûteuse quand l’inspection est inactive.

### Lot I — exemple « Custom Timeline » et exemple combiné

- Implémenter la timeline dans la crate consommatrice de galerie, uniquement
  avec les API publiques : graduation et texte, blocs déplaçables/redimensionnables,
  zoom, scroll, sélection et contrôles Argui standards comme enfants.
- Ajouter commandes clavier/actions, libellés sémantiques, capture correcte,
  thèmes clair/sombre et respect des préférences de mouvement réduit.
- Faire partager son modèle à une liste standard et à une seconde présentation.
  Une modification doit mettre à jour les vues concernées sans partager leur
  sélection locale, leur zoom ou leur focus.
- Démontrer un changement peinture seule et un changement de layout, avec les
  compteurs de phases réels visibles dans les DevTools.

## 3. Ordre d’exécution et portes de validation

Ce plan ne vaut pas autorisation d'annoncer les contrats existants comme finis.
Avant le lot A, relever l'état du worktree et établir une compilation ciblée de
référence : les migrations déjà commencées doivent être achevées dans le chemin
public retenu, sans effacer les changements locaux ni réintroduire d'ancienne API.

1. A : contrat données/présentation et tests consommateurs minimaux.
2. B et C : événements, vrais réveils, cycle de vie, tâches et isolation des montages.
3. D : migration complète et Shared Mailbox fonctionnelle sur les backends concernés.
4. E et F : extension publique, état retenu, feuilles et conteneurs mesurables.
5. G et H : peinture, interaction, accessibilité, invalidation et inspection intégrées.
6. I : timeline et scénario combiné utilisant uniquement les contrats publics.
7. Validation finale, documentation d’utilisation et suppression de tout chemin remplacé.

Chaque lot comporte ses tests avant passage au suivant. Aucun lot n’est déclaré
terminé uniquement parce que son API compile ou qu’une démonstration s’affiche.

### Traçabilité obligatoire

Pendant l'implémentation, attribuer un identifiant à chaque exigence des lots
A–I dans le suivi, puis renseigner son API publique, ses consommateurs migrés,
ses tests et leur résultat réel. Une exigence non vérifiée reste ouverte.

| Sujet demandé | Lots responsables | Preuve attendue |
| --- | --- | --- |
| Modèles sans rendu | A | Modèle métier utilisable sans implémenter `Render` |
| Événements et observations typés | B | Tests de livraison, réentrance, saturation et réveil réel |
| Abonnements et ownership | B, C | Annulation immédiate et absence de rétention des propriétaires |
| Montage, visibilité, démontage, destruction | A, C | Ordre des notifications et nettoyage lors des fermetures réentrantes |
| Services et tâches entre fenêtres | A, C, D | Deux fenêtres réelles partageant les données, pas leur état de présentation |
| Extension sans modification spécifique du moteur | E, I | Timeline définie par un consommateur avec les seules API publiques |
| Mesure et placement personnalisés | F | Feuille et conteneur, enfants ordinaires et contraintes invalides |
| Préparation et peinture | F, G | Commandes graphiques et isolation des caches et clips |
| Interaction et accessibilité | G, H | Pointeur, clavier, capture et actions sémantiques cohérents |
| Invalidation incrémentale | H, I | Compteurs réels : peinture seule sans nouvelle mesure |

Les exemples ne remplacent pas ces tests. Inversement, les tests de logique ne
remplacent pas la vérification des hôtes natifs et Web. La validation finale
doit couvrir les deux, sans page désactivée présentée comme une fonctionnalité.

## 4. Définition de « terminé »

- Chaque exigence des lots A–I est reliée à un test et/ou scénario réel dans
  le suivi d’implémentation, avec résultat et limites de l’environnement de test.
- Tests déterministes sous `tests/`, chemins miroir de `src/` : réentrance,
  annulation, destruction, erreurs, identités, deux montages, layout, commandes
  de peinture, interactions et arbres sémantiques. Pas seulement les parcours réussis.
- Tests consommateurs sans accès privé ; tests de rendu hors écran lorsque le
  GPU est disponible ; scénarios natif/Web, clavier, focus et accessibilité.
  Un test ignoré ou une compilation Web ne vaut pas une validation visuelle.
- Mesurer avant/après : idle, fan-out, drag/scroll/resize, nombre de recalculs,
  durée des phases, ressources vivantes et mémoire après 1 000 cycles de montage.
  Documenter matériel et protocole ; aucun travail de frame au repos, aucune
  croissance des inscriptions/ressources vivantes après nettoyage. Distinguer
  rétention de l’allocateur et fuite réelle ; justifier toute régression mesurée.
- Documenter ownership, ordre des callbacks, erreurs, annulation, limites des
  plateformes, création d’un modèle partagé et création d’une extension complète.
- Respecter les fichiers Rust de 600 lignes maximum, les responsabilités des
  crates et l’absence de shim, de stub ou de dépendance spéculative.
- Pendant le travail, lancer les tests ciblés avec `cargo nextest run --all-features`
  en gardant les mêmes features. Ne pas lancer de couvertures LLVM concurrentes.
- À la fin seulement, lancer `./scripts/quality.sh` exactement une fois juste
  avant le commit autorisé. Respecter le seuil documenté du dépôt : au moins
  90 % séparément sur lignes, fonctions, régions et branches. Si la porte échoue,
  rapporter l’échec et corriger ; ne pas contourner le seuil ni annoncer terminé.

« Aucun écart » signifie ici aucune exigence laissée ouverte ou déguisée en
implémentation. Ce n’est pas une promesse de zéro bug : les preuves, les limites
réelles des plateformes et les vérifications non exécutables restent explicites.
