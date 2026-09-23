# Window safe-area insets

Argui exposes `argui_core::Insets` as renderer-independent window geometry.
`WindowEnvironment::safe_area_insets` makes it available to `AppModel::view`
and `Render` through `Context::environment()`. Values use logical pixels,
matching Argui layout coordinates. `Element::safe_area(insets)` wraps an element
with padding on those four edges, moving the child content into the safe region.
It copies the element's background onto the wrapper so that background can
continue under transparent system bars while text and controls remain inset.
The containing window still needs to draw edge to edge on the platform.

On Android, the Winit host compares `WindowExtAndroid::content_rect()` with the
full drawable window. It samples the rect after each platform event batch
because Winit consumes Android content-rect changes without forwarding a
separate event; updated bars, rotation, or keyboard geometry is picked up as
soon as Android reports it. On iOS, it compares the Winit outer window bounds
with the safe-area bounds returned by `inner_position()` and `inner_size()`.
The renderer and layout viewport use the full iOS outer size; safe-area padding
is applied only when the app calls `Element::safe_area`. Insets are also
refreshed on window creation, resize, scale-factor changes, and redraws. Other
Winit backends default to zero because Winit does not expose a portable
safe-area query there.

`WindowConfig::with_safe_area_insets` supplies an explicit override. An app can
also change or clear the override at runtime with `AppCommand::SetSafeAreaInsets`;
passing `None` restores native detection where supported and zero elsewhere.
This gives browser and custom native adapters the same injection API.

The host exposes the safe rectangle reported by its platform; Argui does not
label the cause of an Android inset or synthesize browser CSS viewport values.
A browser adapter can read those values and pass them through the same command.
Convert physical measurements to logical pixels before injecting them.

```rust,ignore
let environment = cx.environment();
let view = Element::column(children).safe_area(environment.safe_area_insets);
```
