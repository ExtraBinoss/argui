# Mises à jour d'application

`argui-updater` est un moteur sans UI, sans runtime asynchrone imposé et sans
requête réseau implicite. L'application lance une vérification dans sa tâche de
démarrage, puis décide quand télécharger, installer et fermer ses fenêtres.

| Besoin | Feature |
| --- | --- |
| Moteur et backend natif via la façade | `argui/updater` |
| Moteur seul avec backend personnalisé | `argui-updater`, sans feature |
| Backend HTTPS et installation desktop en dépendance directe | `argui-updater/native` |
| Dialogue réutilisable | `argui/widget-updater` ou `argui-widgets/updater` |
| Page de démonstration dans la galerie | `argui-widget-gallery/updater` |

Le dialogue n'active pas le réseau ni l'installation native. Il est explicitement
optionnel, y compris avec `widgets-all`. Le moteur commun et le dialogue compilent
sur WebAssembly ; le backend natif est disponible uniquement hors du navigateur.
Les mises à jour d'une application web restent celles de son déploiement web.

## Au lancement

L'exemple compilable `crates/argui-updater/examples/startup.rs` effectue une
vérification sur un worker et transmet les états par canal, sans installation :

```sh
cargo run -p argui-updater --features native --example startup -- https://updates.example.com/stable/latest.json update.pub
```

```rust,no_run
use argui::updater::{Updater, http::{Config, HttpBackend}, install::NativeInstaller};

let config = Config::new(
    env!("CARGO_PKG_VERSION"), // la version de l'application appelante
    "https://updates.example.com/stable/latest.json",
    include_str!("update.pub"),
)?;
let mut updater = Updater::new(HttpBackend::new(config, NativeInstaller::detect()?)?);
// Dans un worker de démarrage : callback vers le canal d'événements de l'application.
let available = updater.check(|state| { /* transmettre state.clone() à l'UI */ })?;
# Ok::<(), argui::updater::Error>(())
```

Les opérations sont **bloquantes** : ne pas les appeler depuis `Render::render`,
un callback de clic ou directement dans une tâche async. `Context::spawn_blocking`
avec `argui/tasks` permet de les exécuter sur un worker. Conserver son `TaskHandle`
dans le modèle et conserver le moteur retourné par la tâche pour la prochaine
opération. Le callback s'exécute sur le worker : transmettre une copie de `State`
au thread UI, sans capturer d'`Entity` dans le worker. La tâche de démarrage doit
être déclenchée une seule fois, pas à chaque rendu.

Après la vérification, `download(&CancellationToken, callback)` télécharge et
authentifie le paquet ; `install(callback)` effectue l'installation explicitement.
Une nouvelle vérification invalide le téléchargement précédent. L'installation
exige un paquet vérifié et ne peut pas être répétée avec le même paquet. En cas
d'échec d'installation, télécharger de nouveau avant de réessayer.

`CancellationToken::cancel()` peut être appelé depuis le thread UI. Le backend
vérifie l'annulation entre les lectures réseau et le moteur rejette également un
résultat arrivé après l'annulation. Une lecture réseau déjà bloquée peut attendre
le timeout configuré. Créer un nouveau token pour réessayer. Une installation
commencée n'est pas annulable ; fermer le dialogue ne l'interrompt pas.

## Publier une version

L'hébergeur est libre : fichier statique sur un CDN, stockage objet, asset d'une
release GitHub, ou API renvoyant ce JSON. Aucun compte Argui n'est nécessaire.
Pour distinguer les canaux, utiliser des adresses différentes, par exemple
`/stable/latest.json` et `/beta/latest.json`.

```json
{
  "version": "1.2.0",
  "notes": "Démarrage plus rapide et corrections clavier.",
  "platforms": {
    "linux-x86_64": {
      "url": "https://updates.example.com/1.2.0/MyApp.AppImage",
      "signature": "CONTENU COMPLET DU FICHIER MyApp.AppImage.minisig",
      "format": "app-image"
    },
    "macos-aarch64": {
      "url": "https://updates.example.com/1.2.0/MyApp.app.tar.gz",
      "signature": "CONTENU COMPLET DU FICHIER MyApp.app.tar.gz.minisig",
      "format": "app-bundle"
    },
    "windows-x86_64": {
      "url": "https://updates.example.com/1.2.0/MyApp.msi",
      "signature": "CONTENU COMPLET DU FICHIER MyApp.msi.minisig",
      "format": "msi"
    }
  }
}
```

Les clés de plateforme sont `std::env::consts::OS` + `-` +
`std::env::consts::ARCH`. `Config::target` peut préciser un ABI ou une variante
de paquet si plusieurs distributions partagent le même OS et la même architecture.
Une API peut renvoyer `204 No Content` lorsqu'aucune mise à jour n'est disponible.
Un artefact manquant pour la cible constitue une erreur explicite.

La comparaison suit la précédence SemVer : pas de downgrade, pas de mise à jour
pour une différence de métadonnées `+build`. Les préversions sont ignorées par
défaut ; utiliser `Config::allow_prerelease = true` pour un canal de test.

Créer une clé Minisign et signer les **octets exacts** servis par l'hébergeur :

```sh
minisign -G -p update.pub -s update.key
minisign -Sm MyApp.AppImage -s update.key
```

Distribuer `update.pub` avec l'application ; garder la clé privée dans la chaîne
de publication. La valeur `signature` est le contenu texte du `.minisig`, avec
ses retours à la ligne encodés par le sérialiseur JSON. Il ne s'agit pas d'un hash
ni du chemin du fichier. Les signatures Minisign modernes préhachées permettent
la vérification par morceaux. Une signature manquante, ancienne ou invalide
interdit l'installation.

Le backend utilise HTTPS, y compris après redirection ; HTTP est accepté seulement
sur une adresse IP loopback pour les tests. Les URLs contenant des identifiants
sont refusées. Le manifeste est limité à 1 Mio. Les téléchargements sont écrits
dans un fichier temporaire privé, par blocs de 64 Kio, avec une limite par défaut
de 1 Gio. `Config::max_download_bytes` et `Config::timeout` sont configurables
(timeout total de 300 secondes, connexion de 15 secondes maximum).
Les fichiers temporaires incomplets et les paquets abandonnés sont supprimés.

Toutes les dépendances de transport et d'installation appartiennent uniquement
à `argui-updater/native` : Reqwest pour HTTP/TLS, Serde/JSON et SemVer pour le
manifeste, `minisign-verify` pour l'authenticité, `tempfile` pour les fichiers de
staging, `tar`/`flate2` pour les bundles, et `self-replace` pour l'exécutable Windows.

## Installation

| Format | Installation prise en charge |
| --- | --- |
| `executable` | Binaire autonome Windows, Linux ou macOS. Remplacement atomique sur Unix ; `self-replace` gère l'exécutable en cours d'utilisation sous Windows. |
| `app-image` | AppImage Linux brute. `APPIMAGE` identifie le fichier externe au montage ; permissions conservées. |
| `app-bundle` | Archive `.app.tar.gz` contenant un unique bundle du même nom, avec `Contents/MacOS`. Le bundle entier est remplacé, ressources incluses. |
| `nsis` | Installateur Windows `.exe` lancé directement. Son propre mode d'installation et son interface restent ceux configurés par l'éditeur. |
| `msi` | Installateur Windows via `msiexec /i … /passive /norestart`. |

`NativeInstaller::detect()` reconnaît le binaire courant, les AppImage et les
ancêtres `.app`. `Destination` permet de fournir explicitement l'installation.
Le format doit correspondre à cette destination. Les archives de bundle sont
extraites dans un répertoire temporaire sur le même système de fichiers ; le
bundle précédent est restauré si le renommage final échoue. Si la restauration
échoue aussi, l'erreur indique le chemin de la sauvegarde conservée.

Le moteur ne ferme pas le processus et ne lance pas d'élévation. Les permissions
doivent permettre la modification de l'installation. Après `RestartRequired`,
l'application doit sauvegarder son état et redémarrer normalement. Après
`InstallerLaunched`, elle doit sauvegarder puis quitter pour laisser l'installateur
terminer ; le succès du lancement ne signifie pas que l'installation est terminée.
Le fichier temporaire d'un installateur Windows lancé est conservé, puisqu'il
doit survivre au processus appelant ; sa suppression relève du nettoyage des
fichiers temporaires du système.

Pour DEB/RPM, Flatpak, Snap et les stores, fournir un `Installer` ou un `Backend`
adapté au gestionnaire de distribution. Ne pas remplacer directement les fichiers
gérés par ces systèmes. La signature Minisign du téléchargement ne remplace pas
la signature de code ni la notarisation requise par la distribution de l'app.

## Dialogue optionnel

`UpdateDialog::new(key, &state, open, trigger).build(theme)` produit un dialogue
contrôlé, avec les notes de version, le statut, la barre accessible et les quantités
en MB décimaux (`1 MB = 1 000 000 octets`). Une taille absente ou nulle donne une
barre indéterminée et uniquement la quantité déjà reçue. Aucun faux pourcentage
n'est calculé dans ce cas.

`dialog.action(event)` retourne une intention à traiter dans le modèle :

| Action | Traitement |
| --- | --- |
| `Open` / `Close` | Modifier la visibilité du dialogue. |
| `Check` | Lancer `updater.check` sur le worker. |
| `Download` | Créer un token et lancer `updater.download` sur le worker. |
| `Cancel` | Annuler le token de téléchargement. |
| `Install` | Lancer `updater.install` après sauvegarde de l'état de l'application. |

Échap et le clic sur le fond ferment le dialogue. Son action primaire dépend
de l'état : téléchargement, annulation, installation ou nouvelle vérification.
La vérification et l'installation ne proposent aucune action primaire concurrente.
Le dialogue ne contient aucun appel réseau ou système.

Relayer les événements `EventType::Click` et `EventType::Key` vers `dialog.action`
pour traiter aussi Échap. L'exemple de galerie montre le branchement complet.

La galerie fournit une prévisualisation interactive des états, avec une progression
simulée clairement annoncée, et n'installe rien sur la machine :

```sh
cargo run -p argui-widget-gallery --features updater
```

Les tests du moteur utilisent un serveur HTTP local et un artefact signé connu,
sans dépendre d'un hébergeur externe. Les tests d'installation remplacent des
fichiers et bundles temporaires. Les installations Windows et l'exécution d'une
application macOS mise à jour doivent aussi être validées sur ces OS avant une
distribution publique ; les tests exécutés sur Linux ne les remplacent pas.
