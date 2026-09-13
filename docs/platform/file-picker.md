# File picker

La feature `file-picker` expose les dialogues système sans dépendre des widgets :

```toml
argui = { path = "../argui", features = ["file-picker"] }
```

```rust
use argui::platform::file_picker::{FileDialog, FileFilter, FilePickerMode};

let selected = FileDialog::new(FilePickerMode::Files)
    .title("Choisir des images")
    .filter(FileFilter::new("Images", ["png", "jpg", "webp"]))
    .open()
    .await?;

if let Some(files) = selected {
    for file in files {
        // Sur desktop : file.path() fournit un Path, sans conversion UTF-8 imposée.
        println!("{}", file.file_name());
    }
}
```

Modes : `File`, `Files`, `Folder`, `Folders`, `Save`. Les options communes sont
le titre, les filtres d'extensions, le dossier initial et le nom proposé.
`Save` sélectionne une destination ; il n'écrit et ne crée aucun fichier.
L'application reste responsable des opérations d'entrée/sortie et de leurs erreurs.
Les filtres guident la sélection ; ils ne valident pas le contenu d'un fichier.

`FileDialog::parent(Arc<W>)` associe une fenêtre implémentant les traits
`HasWindowHandle` et `HasDisplayHandle`. Le propriétaire est conservé jusqu'à
la fin du dialogue natif, même si le consommateur abandonne son résultat.
L'API fonctionne aussi sans parent explicite ; le widget utilise alors un
dialogue autonome. Passer un parent dans sa configuration pour une relation modale.

`FileDialogBackend` est l'interface commune. `NativeFileDialog`, le fournisseur
par défaut, utilise [RFD 0.17.2](https://docs.rs/rfd/0.17.2/rfd/), qui possède déjà
les adaptateurs OS. Une application peut fournir son propre backend via
`open_with`, ou `.backend(...)` sur le widget.

| Système | Dialogue |
| --- | --- |
| Linux Wayland / X11 | XDG Desktop Portal et le sélecteur du bureau ; RFD peut utiliser Zenity en repli. Installer un backend de portail avec FileChooser et Zenity pour ce repli. |
| Windows | Dialogues COM `IFileOpenDialog` / `IFileSaveDialog`. |
| macOS | Panneaux AppKit `NSOpenPanel` / `NSSavePanel`. L'application et sa boucle d'événements doivent fonctionner sur le thread principal. |
| Web | Sélection simple/multiple via le sélecteur de fichiers du navigateur. Les dossiers et destinations d'enregistrement retournent `UnsupportedMode`. |

Les appels ne bloquent pas la boucle Argui. Sur le web, lancer depuis une action
utilisateur pour conserver l'autorisation d'ouverture du navigateur. Sur desktop,
le travail conserve les handles natifs jusqu'à la réponse du système.
Abandonner la future annule la réception du résultat, pas nécessairement le
panneau déjà présenté par le système.

`FileDialogResult` distingue une sélection, une absence de sélection et une
configuration ou un lancement impossible. **RFD ne distingue pas l'annulation
utilisateur de tous les échecs internes du dialogue** : `Ok(None)` veut dire
« aucun fichier retourné », et ne constitue pas une preuve d'annulation.
Les chaînes contenant NUL, les noms qui sont des chemins et les filtres invalides
sont rejetés avant tout appel natif.

## Widget

```toml
argui = { path = "../argui", features = ["widget-file-picker"] }
```

`widgets-all` et `argui-widgets/all` l'incluent ; toutes ces features sont
inactives par défaut. L'API seule ne charge pas les widgets ni leurs tâches.

```rust
use argui::{
    platform::file_picker::{FileDialog, FilePickerMode},
    widgets::FilePicker,
};

// Dans le contexte d'un composant, conserver cette Entity dans son état :
let picker = cx.new_entity(FilePicker::new(
    "project-folder",
    "Choisir le projet",
    FileDialog::new(FilePickerMode::Folder).title("Dossier du projet"),
));
// Puis, dans render :
let element = cx.entity(&picker);
```

Le composant installe le clic, l'activation clavier et l'accessibilité. Il montre
les sélections réelles et les erreurs, bloque les doubles ouvertures et conserve
la sélection précédente quand aucun nouveau choix n'est retourné.
`selection()`, `status()` et `is_open()` permettent de lire son état ; `open(cx)`
permet aussi de le déclencher depuis une commande. `build(theme, cx)` accepte
un thème explicite. `FilePickerEvent::{Selected, Dismissed, Failed}` s'écoute avec
`cx.subscribe(&picker, callback)` ; conserver la `Subscription` retournée.
Les abonnés et le composant doivent appartenir au même `ModelRuntime`.

La galerie contient **File picker** dans la liste alphabétique, avec cinq
exemples : document, images multiples, dossier, dossiers multiples et destination
d'export. Le dernier exemple ne modifie pas le fichier choisi.

## Validation

Les tests isolent le fournisseur natif : propagation des options, validations,
sélection réelle, fermeture sans changement, erreurs, double activation et absence
d'exécuteur. Ils ne prennent pas de captures du bureau. L'ouverture visuelle et
le choix des fichiers se vérifient manuellement dans la galerie sur chaque OS.
