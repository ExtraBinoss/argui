# Documentation Argui

Les guides décrivent les contrats actuels et leurs limites. Les anciens plans de
livraison et bilans de chantier restent consultables dans l'historique Git.
Les commandes se lancent depuis la racine du dépôt.

Commencer par l'[architecture](architecture.md), le [catalogue des widgets](widgets/shadcn.md)
ou les [modèles et présentations](runtime/models.md). La [roadmap](roadmap.md)
contient uniquement les travaux ouverts. Les exemples complets sont dans la
[Widget Gallery](../crates/argui-widget-gallery/src/pages/).

| Sujet | Guides |
| --- | --- |
| Runtime | [Modèles, ownership, cycle de vie et services](runtime/models.md) · [Tâches asynchrones](runtime/tasks.md) |
| UI | [Interaction, focus et accessibilité](ui/interaction.md) · [Styles et thèmes](ui/styling.md) · [Animation](ui/animation.md) · [Actions et édition](ui/editing.md) · [Scroll et virtualisation](ui/scroll.md) · [Éléments personnalisés](ui/custom-elements.md) |
| Widgets | [Catalogue shadcn et contrats de composition](widgets/shadcn.md) · [Listes et tables](widgets/lists-tables.md) · [Overlays et placement](widgets/overlays.md) |
| Plateformes | [Application, fenêtres et tray](platform/application.md) · [WebView](platform/webview.md) · [Sélection de fichiers](platform/file-picker.md) · [Fonds de bureau](platform/desktop-backdrops.md) · [Popovers natifs](platform/native-popovers.md) · [Mises à jour](platform/updater.md) |
| Rendu | [Primitives, couleurs, images et SVG](rendering/primitives.md) · [Effets GPU](rendering/effects.md) |
| Performance | [Optimisations CPU/RAM et mesures](performance/optimizations.md) · [Empreinte des applications et de la galerie](performance/footprint.md) |
| Contribution | [Qualité, dépendances et préparation](contributing/code-quality.md) · [Tests graphiques Linux](contributing/linux-testing.md) · [DevTools et profiling](contributing/devtools.md) |

Les mesures de performance conservent leurs conditions, baselines et limites.
Leurs [données brutes](performance/data/) sont séparées des guides. Les résultats
d'une ancienne campagne ne remplacent pas une validation de la version courante.
