# Widget interaction APIs

Every widget remains controlled: handlers report intent or a next value, and
the application supplies the resulting state on the next render. Direct
handlers are additive and use the same capture/target/bubble dispatch pipeline
as `Element::on`.

## Direct handlers

| Widget | Direct API and payload | Keyboard and accessibility contract |
| --- | --- | --- |
| Button | `on_click(EventHandler)` | Enter, Space, pointer, touch, and accessibility click converge. Disabled or busy buttons do not emit. |
| Input, TextArea | `on_edit(TextEdit)`, `on_input(String)`, `on_submit(String)` | Editing, paste, IME, Enter submission, and accessibility set-value use the text-input policy. `on_edit` avoids cloning a large complete value. |
| InputOtp | `on_input(String)`, `on_submit(String)` | Numeric filtering happens before delivery; completion submits the controlled value. |
| Checkbox | `on_change(CheckedState)` | Click/Space/accessibility click; Mixed advances to Checked. |
| Switch, Toggle | `on_change(bool)` | Click/Space/accessibility click; disabled controls do not emit. |
| ToggleGroup | `on_change(Vec<String>)` | Roving focus and activation emit the next stable selection IDs. |
| RadioGroup, Tabs | `on_select(usize)` | Arrow navigation and activation emit the source index. |
| Select, NativeSelect | `on_select(usize)`, `on_open_change(bool)` | Disabled options are skipped; Escape and outside dismissal request close. |
| Combobox | `on_input(String)`, `on_select(usize)`, `on_open_change(bool)` | Active-descendant navigation and Enter selection share the same payloads. |
| Slider | `on_change(f32)`, `on_commit(f32)` | Pointer pan, arrows, Home/End, and accessibility value changes use the configured bounds and step. |
| ColorPicker | `on_change(Color)` | Pad, hue, alpha, and text-field edits produce the next complete color. |
| SplitPane | `on_change(f32)` | Drag, double click, arrows, Home, and End produce the next constrained pane size. |
| Collapsible | `on_open_change(bool)` | Trigger click, Enter, and Space request the next open state. |
| Accordion | `on_open_change(Vec<String>)` | Arrow/Home/End navigation plus activation emit the next open section IDs. |
| Dialog, Sheet, Drawer | `on_open_change(bool)`, `on_dismiss(EventHandler)` | Trapped modal focus, Escape, close controls, gestures, and configured outside behavior remain unified. |
| AlertDialog | `on_open_change(bool)`, `on_action(String)` | Initial cancel focus and confirmation/cancel activation emit stable action IDs. |
| Popover, HoverCard | `on_open_change(bool)`, `on_dismiss(EventHandler)` | Trigger/focus/hover rules and Escape/outside dismissal request controlled state. |
| Menu, ContextMenu, Menubar | `on_action(String)`, `on_open_change(bool)` | Roving focus, typeahead, submenus, Enter/Space, context-key, and Escape preserve menu behavior. |
| CommandPalette | `on_input(String)`, `on_action(String)`, `on_open_change(bool)` | Search and active-command keyboard navigation emit stable command IDs. |
| Calendar | `on_select(String)` | Day activation emits ISO `YYYY-MM-DD`; constrained days never emit. Month and year navigation remain controlled through `CalendarState`. |
| DatePicker | `on_input(String)`, `on_select(String)`, `on_open_change(bool)` | Input validation, compact calendar/year navigation, keyboard behavior, and dismissal remain controlled. |
| Pagination, Carousel | `on_select(usize)` | Buttons, arrows, swipe (carousel), and accessibility activation emit the next source index/page. |
| Breadcrumb | `on_activate(String)` | Actionable ancestors emit their stable IDs. |
| List, VList, Table, TreeView | `on_select(String)`, `on_activate(String)` | Pointer, Enter/Space, arrows, Home/End, typeahead, and accessibility actions emit stable row/node IDs. |
| DataTable | `on_select(String)`, `on_activate(Vec<String>)` | Selection emits a row ID; activation emits `[row_id, column_id]`. Editing/sort/resize stay available through `DataTableAction`. |
| Chart | `on_activate(Vec<usize>)` | Pointer and accessible points emit `[series_index, point_index]`. |
| NavigationMenu | `on_action(String)`, `on_open_change(String)` | Stable item IDs are emitted; an empty open value means closed. |
| Questionnaire | `on_input(String)`, `on_choice(Vec<String>)`, `on_navigate(usize)`, `on_submit(EventHandler)` | Choice emits `[question_id, choice_id, selected_bool]`; controlled answers and validation stay in the model. |
| MessageScroller | `on_load_earlier(EventHandler)`, `on_latest(EventHandler)` | Buttons and accessible activation preserve scroll/follow-bottom behavior. |
| ToastHost | `on_close(String)` | Close buttons emit the toast ID; hover/focus pause remains available through `pause_action`. |
| Sidebar | `on_collapsed_change(bool)`, `on_open_change(bool)` | Rail and mobile sheet controls emit the requested controlled state. |
| UpdateDialog | `on_action(String)`, `on_open_change(bool)` | Emits `check`, `download`, `cancel`, or `install`; engine state stays controlled. |

## Incremental text edits

Use `on_input(cx.input_callback(...))` when the complete next value is the most
convenient payload. For document-sized buffers, use
`on_edit(cx.edit_callback(...))`: `TextEdit` contains the half-open UTF-8 byte
range in the previous value and its replacement text.

```rust,ignore
TextArea::new("source", &self.source, "Start writing…", theme.input())
    .on_edit(cx.edit_callback(|model, edit| {
        edit.apply_to(&mut model.source)
            .expect("an editor edit matches the controlled revision");
    }))
    .build()
```

The engine emits both event forms only when both are observed. A control with
only an `on_edit` handler does not allocate or copy its complete value for
delivery. `Context::edit_event_handler` is the non-invalidating counterpart for
apps that update retained text immediately and defer expensive derived work.
The learning site's live Interaction API example shows the delivered byte range
and replacement length after every edit. [Astra Editor](../../app_examples/astra-editor/README.md)
uses the same path for its code buffer and includes reproducible
incremental-delivery and native latency profiles.

## Intentionally action-only or presentational

| Widget | Reason |
| --- | --- |
| RangeBehavior and typed behavior/action types | They are the runtime-neutral advanced layer used to build Slider and custom controls. |
| DataTable editing, sorting, filtering, resizing | These operations mutate a typed table model and can carry validation/focus effects; `DataTableAction` is clearer than flattening them into unrelated callbacks. |
| Complex multi-selection reducers | Modifier/range selection needs the complete action and prior controlled state; List/Table behavior APIs remain available alongside simple direct selection. |
| FilePicker | Opening the OS picker is an asynchronous platform capability with cancellation/error results, not a synchronous element value. |
| Tooltip | Hover/focus delays and hoverable-content transitions are temporary interaction state; `TooltipBehavior` remains the explicit controller. |
| Label | Its click is a focus request for another key, handled through `Label::focus_target` and `Context::request_focus`. |
| ScrollArea | Wheel, gesture, keyboard, scrollbar, and programmatic requests are scroll-engine operations rather than business-data callbacks. |
| Attachment | File operations are application-defined child actions; the widget only presents controlled metadata and progress. |
| Alert, AnimatedText, AspectRatio, Avatar, Badge, Bubble, ButtonGroup, Card, Direction, Empty, Field, InputGroup, Item, Kbd, Marker, Message, Progress, Separator, Skeleton, Spinner, TextSelection, Typography | These widgets are presentational or compose interactive children. They intentionally do not invent a business action. |
| Icons and theme/assets helpers | These produce presentation resources, not controls. |

## Testing pattern

Prefer an accessible query and assert the controlled result:

```rust,ignore
let mut app = TestApp::new(Settings::default());
app.get_by_role(Role::CheckBox, "Email alerts").click()?;
app.assert_state("email-alerts", SemanticMatcher::Checked);
# Ok::<(), argui_testing::TestError>(())
```

For advanced reducers, keep a focused behavior test for every action and add one
application test proving that layout, input routing, focus, and model
reconciliation work together. See [lists and tables](lists-tables.md),
[overlays](overlays.md), and [interaction](../ui/interaction.md).
