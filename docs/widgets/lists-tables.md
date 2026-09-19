# Lists and tables

Enable `widget-list`, `widget-vlist` or `widget-table` on the `argui` facade,
or use `widgets-all`. The gallery has separate List, VList and Table examples.

## List

```rust
use argui::ui::Element;
use argui::widgets::{Collection, CollectionItem, List, ListState};

// Retain this state in the owning component.
let selection = ListState::default();
let items = Collection::new((0..8).map(|i| CollectionItem::new(i.to_string(), format!("Item {i}")))).unwrap();
let list = List::new("items", &items)
    .label("Items")
    .selection(&selection, true);
let element = list.build(theme, |index| Element::text(format!("Item {index}")));
```

`List::action(&event)` returns the next controlled state. Attach Click and Key
listeners to the root. For a recognized action, prevent default keyboard
handling, request focus on the list container (`"items"`) and notify the
component. Keyed interactive descendants keep their own behavior and do not
select the row. Explicit content labels are preserved; simple text rows get
an accessible name automatically.

Click selects, Ctrl/Cmd-click toggles, Shift extends from the anchor and
Ctrl/Cmd-Shift adds a range. Arrow keys and Home/End move the active row;
Shift extends selection and Ctrl/Cmd moves without selecting. Enter or Space
activates the row; Ctrl/Cmd-A selects all in multiple-selection mode.

## Variable-height virtual lists

```rust
use argui::ui::VirtualList;
use argui::widgets::VList;

// Create once and retain it so measurements survive rendering.
let heights = VirtualList::variable(10_000, 40.0, 320.0);
let list = VList::variable("items", &heights, offset);
let element = list.build_list(&items, &selection, true, theme, |index| {
    Element::text(format!("Item {index}"))
});
```

Layout measures mounted rows, and cloned models share those measurements. The
engine corrects the scroll anchor when a row above the viewport changes height.
The initial height is an estimate. `VList::new` provides fixed heights; `build`
is available without selection. The widget's row count must match its retained
model or construction fails immediately.

Keep the offset from Scroll events. To reach an unmounted active row, compute
`heights.scroll_to(items.index_of(active).unwrap(), VirtualAlignment::Nearest, offset)`,
rebuild the visible window and send `ScrollRequest::offset` with the focus request.
`heights.with_viewport(new_height)` preserves measurements on resize.

`Collection` builds an index of stable IDs when data changes. Selection and its
anchor follow IDs after sorting. Call `ListState::reconcile` against the full
dataset before filtering. Height measurements remain positional: apply matching
`VirtualList::insert/remove` operations when data changes.

Focus stays on the container, with `active_descendant` identifying the active
option. That option remains mounted offscreen; rendering is bounded by the
visible window plus that row. `List::page_size` enables PageUp/PageDown.
`List::search` uses retained `Typeahead`, caller-supplied monotonic time and an
optional custom matcher.

## Table

```rust
use argui::widgets::{Table, TableColumn};

let table = Table::new("values", [
    TableColumn::new("Name", 180.0),
    TableColumn::new("Value", 120.0),
], &items).label("Values").selection(&selection, true);
let element = table.build(theme, |row, column| {
    Element::text(format!("{row}:{column}"))
});
```

`Table::action` follows the List protocol. Columns have explicit positive widths;
headers and rows share them. Interactive tables expose Grid/Row/ColumnHeader/Cell
semantics and row selection. The widget styles headers and row states; apply
text styles to cell content with the theme. Wrap wide tables in a horizontal
viewport.

`Table` mounts all rows. Sorting, pagination, virtualization and cell editing
belong to `DataTable`. Semantic tests run without a display; real native and web
screen-reader validation remains separate.

See the [controlled integration](../../crates/argui-widget-gallery/src/pages/data.rs)
and [list](../../crates/argui-widgets/tests/list.rs),
[VList](../../crates/argui-widgets/tests/vlist.rs),
[table](../../crates/argui-widgets/tests/table.rs) and
[gallery tests](../../crates/argui-widget-gallery/tests/pages/data.rs).

## DataTable

`widget-data-table` exposes `DataTableModel<R>`, `DataColumn<R>` and `DataTable`.
Columns accept typed callbacks for values, comparison, filtering, rendering,
validation and editors. Explicit changes recompute filtering, stable multi-column
sorting and pagination, in that order. The widget borrows the model and a
`VirtualList` matching the current page. Visible columns share header widths.
Read `selection()` and `edit()`; change them through `set_selection`, `begin_edit`
or `apply` to preserve validation and caches.

An editor keeps a draft. `commit_edit` or `apply(CommitEdit)` returns `CellCommit`
for the owner to apply; persistence is never implicit. Validation errors keep
the editor open. Escape cancels; Tab commits and moves the active cell. Removal,
filtering or an external change to the original value cancels editing.

Before a transformation, retain `model.collection()` and then call
`model.collection().remap_heights(&previous, &mut heights)` so measurements follow
stable IDs. The collection snapshot is shared without traversing rows. Column
handles reuse SplitPane gestures and keyboard behavior: forward Gesture events
and apply `ResizeColumn`. Their visible separator remains compact while hit slop
provides a 24-pixel pointer target; captured resize drags do not leak into the
table's horizontal touch scroll.

DataTable owns its horizontal viewport, keeping headers and rows aligned after
resizing. The vertical viewport contains only rows and preserves virtualization.

## TreeView focus

One visible selected row, or the first row when nothing is selected, participates
in Tab navigation. Other rows accept programmatic focus. The active row remains
mounted outside the virtual window, and the cache holds at most that row plus
the visible rows. Tab leaves the tree; PageUp/PageDown moves by a visible page.
The owner applies `TreeAction` and requests focus on the new key.
