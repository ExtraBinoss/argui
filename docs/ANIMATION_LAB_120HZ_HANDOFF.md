# Passation performance — galerie Solid/React native (24 septembre 2026)

## Où reprendre

- Branche : `codex/dsl-gallery-live`.
- Worktree : `/home/albi/.codex/worktrees/22aa/argui`.
- Un commit de checkpoint de la migration DSL vers Solid/React/QuickJS est demandé malgré la performance et le gate encore incomplets. Reprendre ce checkpoint sans écraser le worktree.
- Lire `AGENTS.md`, `docs/contributing/code-quality.md`, `docs/contributing/linux-testing.md`, puis `docs/SOLID_REACT_HANDOFF.md`.
- L'APK présent sur le Pixel 8a est la galerie Solid **normale et interactive**, sans option `-ParguiComparison`. Il démarre sur Button. Le mode de comparaison `rust` fige volontairement les callbacks JS : ne pas le laisser installé pour tester les clics.

## Résultat réel aujourd'hui

La galerie est utilisable : Button, Select, Animation Lab et Media s'ouvrent sur Android. Select affiche maintenant la valeur choisie dans le champ et dans le résumé. Les boucles d'Animation Lab fonctionnent en natif et le spinner tourne. Les pages Solid et React ont le même ordre et le même contenu ; l'habillage est responsive. Sur Android, Travel est le premier exemple d'Animation Lab, puis viennent les autres cartes dont Opacity. La navigation reste hors du viewport scrollable et une page nouvellement ouverte repart en haut.

La performance **n'atteint pas encore 120 fps constants**. Sur le Pixel 8a à 120 Hz, le spinner Button isolé a donné une médiane SurfaceFlinger d'environ 8,4 ms et un p95 d'environ 16,8 ms, sans intervalle au-dessus de 25 ms dans cet échantillon. Sur l'APK normal, deux gestes Animation Lab depuis le haut ont donné respectivement p50/p95 de 16,7/41,8 ms et 25,0/50,1 ms. Les variations de position, charge et température comptent ; aucune de ces mesures ne prouve 60 fps constants, encore moins 120 fps. Le ressenti utilisateur d'un scroll saccadé reste cohérent avec ces intervalles.

Les captures non blanches du build Android final sont `/tmp/argui-normal-button.png`, `/tmp/argui-normal-webgpu.png`, `/tmp/argui-normal-media.png` et `/tmp/argui-normal-lab.png`. Les vérifications desktop utilisent l'affichage privé imposé par `docs/contributing/linux-testing.md`; Select et Media ont des captures privées validées. La fluidité desktop complète n'a pas été certifiée à 120 Hz.

## Ce qui a été corrigé et vérifié

| Zone | Défaut observé | Correction et preuve |
| --- | --- | --- |
| Pont TSX → Host | Les recherches linéaires de propriétés/événements pénalisaient le remontage de page. | Cache des noms valides ; sur Pixel Animation Lab → Button, callback d'environ 106 à 43 ms et premier rendu d'environ 95 à 55 ms entre les essais. Les opérations et la scène restent identiques. |
| Select | L'état et le résumé passaient à WebGPU, mais le champ restait à Vulkan. | Le Host ne réutilise plus un Rectangle si l'identité de ses enfants matérialisés a changé. Test rouge puis vert ; captures desktop privée et Android final montrent `WebGPU` et `Selected WebGPU`. |
| Media | Button → Media pouvait prendre presque une seconde. | Un SVG de 964 × 643 était rasterisé trois fois pendant la croissance de l'atlas 256 → 512 → 1024. Contrôle de capacité avant rasterisation ; illustration décorative pré-rasterisée en PNG depuis son SVG source à la construction. Les icônes Tabler restent de vrais SVG natifs. Premier rendu Android mesuré d'environ 886 à 108 ms entre builds. |
| Boucles Size/Gap | Leur layout changeait continuellement même sans interaction. | Boucles visuelles en transforms natives ; `Change target` conserve les vraies transitions de largeur/gap. Le temps de tree idle mesuré sur le profil comparable est tombé d'environ 3 ms à 0 ms. |
| Scroll/layout | La navigation défilait avec le contenu et le layout pouvait se casser. | Navigation sœur du viewport scrollable sur mobile et desktop ; scroll réinitialisé par page. Captures Android finales non blanches et structure vérifiée. |
| Renderer | Textures inactives évincées après 60 images même sous le budget, ce qui invalidait tout le root retenu. | Conservation tant que le budget est respecté, test de régression sur 61 images ; plus de repaint complet périodique dans deux séries de 120 images après démarrage. |
| Passes GPU | Clears séparés et clones de `DrawBatch` augmentaient le travail natif. | Fusion de clear avec premier draw lorsqu'elle préserve la couleur exacte ; chemin emprunté pour les runs Draw contigus. Tests renderer 56/56 et stress GPU X11 privé validés. Gain de scroll variable, pas une résolution du problème. |
| Desktop | Le chemin Wayland du test privé avait des acquisitions de surface proches d'une seconde. | Préférence X11 lorsqu'un `DISPLAY` existe, avec fallback Wayland. Sur X11 privé, les navigations testées n'avaient plus ces timeouts. Ne pas confondre ce problème de backend avec le coût de scroll encore présent. |

L'ancienne galerie Rust et la nouvelle galerie Solid passent toutes deux par le scroll natif d'Argui (`Overflow::Auto` / `scroll_y` vers `scroll_or_exit`). Nous n'avons **pas identifié un ancien gestionnaire de scroll supprimé** qui expliquerait à lui seul la différence ressentie. L'ancienne scène et la scène TSX actuelle n'ont pas la même complexité, les mêmes couches ou les mêmes effets. L'ancienne galerie avait elle aussi des animations de rayon/layout ; dire qu'elle était entièrement compositée serait faux.

## Ce que la comparaison Rust ↔ QuickJS prouve

Le runner de comparaison instancie **la même scène Animation Lab** de deux façons : Solid/QuickJS vivant, ou snapshot validé `Element` injecté dans un `AppModel` Rust direct. Le renderer, le viewport, les assets et les boucles natives sont les mêmes. Dans un échantillon Pixel, les médianes idle étaient toutes deux autour de 15,3 ms ; les médianes de scroll étaient 18,15 ms en QuickJS et 18,62 ms en Rust. Après le démarrage, le mode QuickJS a émis zéro batch JS pendant l'animation mesurée. Ces chiffres sont des échantillons, pas une promesse de cadence ; ils montrent que QuickJS n'est pas le moteur du stutter continu.

Le callback JS contribue encore au **changement de page** (d'où le cache Host), mais pas à chaque image du spinner ou du scroll. Pour comparer à nouveau, suivre `apps/gallery/README.md` et reconstruire explicitement chaque variante. Laisser ensuite l'APK normal sur le téléphone.

## Pourquoi Animation Lab scrolle encore mal

Un geste scroll déplace les bounds de nombreuses couches d'effets. Le cache actuel du foreground exige même région écran, mêmes bounds et même révision (`crates/argui-render/src/surface/effects/helpers.rs`, `crates/argui-render/src/surface/effects.rs`). Le déplacement suffit donc à faire manquer la réutilisation. Le renderer passe par `render_effect_graph` et réencode les draws ; les images Full/Seed peuvent concerner les 1080 × 2400 pixels du Pixel. Les profils observés comptaient environ 19 couches d'effets, 27–29 batches, et environ 15–30 ms d'encodage CPU sur les images de scroll coûteuses. Les trois régions de dommage ne couvraient parfois que 140–160 k pixels, mais leur union clippée atteignait environ 570–584 k pixels. Le coût dépend aussi du nombre de couches réellement visibles.

Un A/B avec `Pause loops` **confirmé par le bouton `Resume loops`** conservait une médiane de scroll proche de 25 ms (25,02 ms actif, 25,07 ms en pause). Les boucles ne sont donc pas la cause dominante du geste. Un cache strict de translation pour les seules couches triviales a été évalué puis abandonné : sur la scène mobile 393 × 852, seulement **1 des 14 couches** offscreen visibles était éligible ; sur desktop 1200 × 800, 8 sur 20. Le prototype temporaire et ses dépendances ont été retirés. Un simple changement de comparaison des bounds ou un déplacement aveugle de texture serait incorrect avec clips, tuiles, filtres, masks et coordonnées fractionnaires.

Le précédent essai de copie de scroll avait corrompu le layout et a été retiré avant cette passation. Ne pas le réactiver tel quel.

## Plan concret pour viser 120 Hz sur Android et desktop

Le budget d'une image à 120 Hz est **8,33 ms entre présentations** ; à 60 Hz, 16,67 ms. Les durées CPU, GPU et SurfaceFlinger ne sont pas additionnables telles quelles, car elles couvrent des étapes différentes et peuvent se chevaucher. L'acceptation doit venir des timestamps présentés et de captures visuellement correctes, pas seulement d'un compteur de frames produit par l'app.

1. **Figer une baseline reproductible.** Tester l'APK normal, sans télémétrie de comparaison, après échauffement contrôlé, avec le même viewport, la même position de départ et le même geste depuis le haut. Capturer SurfaceFlinger p50/p95/p99 et nombre d'intervalles au-dessus de 8,33/16,67 ms ; noter état thermique. Sur desktop, faire le même geste sur l'affichage privé X11, puis vérifier Wayland séparément. Conserver la comparaison Rust/QuickJS de la même scène comme contrôle : si les deux ralentissent pareil, travailler dans le renderer.
2. **Séparer contenu scrollable et position écran.** Introduire une représentation retenue du contenu en coordonnées locales du scroll, avec clé de sous-arbre stable et révision de contenu. Le `scroll_offset` doit modifier une transformation de composition, pas l'identité du contenu rasterisé. Une carte statique qui change seulement de position doit réutiliser ses pixels ; couleur, texte, thème, taille, scale factor, clip local ou effet modifié doivent invalider exactement la carte concernée.
3. **Mettre le contenu statique en tuiles bornées.** Rasteriser les cartes et le texte statiques en tuiles de taille fixe avec un léger overscan ; réutiliser les tuiles visibles pendant le drag, et rasteriser seulement la bande nouvellement exposée. Composer les tuiles à leur position du frame courant avec un clip de viewport correct. Garder un budget mémoire explicite et une éviction LRU **par tuile**, sans vider le root ni les autres caches lors d'une seule éviction. Ce plan remplace la copie du framebuffer qui avait cassé le rendu.
4. **Superposer les parties animées.** Les translations/rotations/opacités natives se composent sur des couches locales au lieu de re-rasteriser les tuiles statiques. Les propriétés réellement paint (rayon, couleur, texte) ne réactualisent que leur petite couche visible. Les animations hors viewport ne doivent pas forcer l'encodage des cartes invisibles. Éviter qu'une seule animation Paint impose à toutes les couches voisines un redraw complet.
5. **Réduire l'encodage CPU résiduel.** Profiler séparément la préparation du texte, la création des passes, les appels `draw_offscreen`, le submit et le GPU. Les premiers profils donnent environ 1,5–5 ms de texte selon la portion visible, mais le plus gros poste est l'encodage des 19 couches/27–29 batches. Si les tuiles réutilisées traversent encore le même encodage, conserver leurs commandes GPU préparées ou des `RenderBundle` par tuile valide ; mettre à jour uniquement offset/clip et draws animés. Mesurer avant de généraliser cette optimisation.
6. **Construire des tests de justesse avant d'activer le fast path.** Comparer pixel à pixel le chemin retenu et le redraw complet après séquences de scroll entier/fractionnaire, retour arrière, resize, DPI, nested clips, coins arrondis, ombres, transparence, blur/backdrop, thème, changement de texte, Media et interactions. Pour tout effet non prouvé sûr, revenir au renderer existant. Sur Linux, utiliser `./scripts/linux-hidden-display.sh`, captures non blanches et stress GPU ; sur Pixel, vérifier captures, clics et scroll après plusieurs allers-retours.
7. **N'accepter la refonte qu'avec l'A/B produit.** Même scène et même geste : aucune corruption, navigation/Select/Media inchangés, puis p95 des intervalles présentés proche ou sous 8,33 ms pour 120 Hz sur le Pixel visé et sur un desktop 120 Hz. Si le matériel/backend limite ce seuil, isoler la phase bloquante et publier la cadence réellement mesurée ; ne pas déclarer « fully smooth » sur la seule moyenne ou sur un écran 60 Hz.

Les premiers fichiers à lire sont `crates/argui-runtime/src/app/scroll.rs`, `crates/argui-runtime/src/app/renderer.rs`, `crates/argui-render/src/surface.rs`, `crates/argui-render/src/surface/effect_damage.rs`, `crates/argui-render/src/surface/effects.rs`, `crates/argui-render/src/surface/effects/helpers.rs`, `crates/argui-render/src/effect_graph.rs` et `crates/argui-layout/src/paint/cull.rs`. Garder le renderer indépendant du DSL retiré.

## Hot reload et autres limites

Il n'y a **pas de hot reload natif Solid/React aujourd'hui**. `package.json` lance `vite build` ; `apps/gallery/quickjs-host/src/runner.rs` embarque `gallery-core.mjs` par `include_str!`, et Gradle reconstruit le bundle et l'APK. Une édition TSX demande donc bundle + relance du binaire desktop ou réinstallation Android. **L'utilisateur demande explicitement d'implémenter le hot reload à la prochaine reprise.** Une boucle de développement devrait charger un bundle depuis un fichier, observer sa reconstruction, puis remplacer proprement le module QuickJS et son arbre Host à la frontière d'un frame. Elle doit préserver la dernière scène valide sur erreur de compilation, retirer anciens listeners/timers, et réinitialiser explicitement les états qui ne peuvent pas être migrés. Prévoir sa validation sur desktop et Android.

Le comportement iOS et TalkBack n'a pas été revérifié dans cette session. La vidéo native n'est pas intégrée. Les échantillons de performance ne certifient pas tous les appareils.

## Vérifications effectuées et état du gate

- Tests ciblés Host 13/13 et renderer 56/56 avec `--all-features` ; tests de galerie/parité Solid/React et TypeScript passés.
- Clippy ciblé Host/renderer, stress GPU X11 privé, `git diff --check`, taille des fichiers Rust et structure miroir des tests passés.
- Préflight statique complet (`check-quality-static.sh`) passé : formatage, Clippy workspace, WASM et contrats API.
- L'APK Android normal a été réinstallé puis testé : Button initial, Select → WebGPU, Media, Animation Lab et captures non blanches.
- `quality.sh` a été lancé une fois avec `ARGUI_NATIVE_TESTS=1`, puis s'est arrêté pendant la compilation instrumentée de couverture (`argui-render`/test `offscreen`) ; aucun rapport de couverture final n'a été produit. Erreur précise : `E0658`, `#[cfg_attr(coverage_nightly, coverage(off))]` dans `crates/argui-render/src/offscreen.rs:39` est compilé par le test d'intégration qui inclut ce source sans l'attribut de crate `#![feature(coverage_attribute)]`. **Ne pas annoncer le gate vert.** L'utilisateur demande un commit de checkpoint malgré cet échec et le travail 120 Hz restant ; réparer le gate lors de la reprise.

La prochaine reprise doit traiter la refonte du scroll décrite ci-dessus et le hot reload Solid/React demandé par l'utilisateur. Mesurer à nouveau sur desktop et Android, corriger le gate puis effectuer le commit final une fois les objectifs vérifiés.

## Consigne prête à transmettre au prochain agent

> Reprends `codex/dsl-gallery-live` dans `/home/albi/.codex/worktrees/22aa/argui`. Lis `AGENTS.md`, `docs/SOLID_REACT_HANDOFF.md` et ce document. Le checkpoint est incomplet : Animation Lab scroll encore bien au-dessus de 8,33 ms p95 sur Pixel 8a, le desktop 120 Hz n'est pas certifié, et `quality.sh` échoue sur E0658 dans le test `offscreen`. Termine l'audit et la refonte du rendu/cache natif en comparant la même scène Rust et TSX/QuickJS ; conserve la parité mobile/desktop, les clics et les boucles natives. Implémente aussi le hot reload natif Solid/React demandé pour la prochaine reprise, sur desktop et Android. Valide la justesse visuelle, les interactions, le 120 Hz mesuré et le gate qualité avant le commit final.
