# Widget builder inventory

The 0.3 API audit applies one ownership rule throughout the widget crate:
constructors receive controlled application data, chainable methods configure
optional behavior, and `build` produces the element. Direct handlers are
additive configuration and never replace controlled state.

## Consistent families

| Build shape | Widgets | Reason |
| --- | --- | --- |
| `build(theme)` | Accordion, Alert, AlertDialog, Attachment, Avatar, Badge, Breadcrumb, Bubble, Calendar, Card, Carousel, Collapsible, ColorPicker, Combobox, DataTable, DatePicker, Dialog, Drawer, Empty, Field, HoverCard, InputOtp, Item, Kbd, Label, List, Marker, MessageScroller, NativeSelect, NavigationMenu, Pagination, Popover, Progress, Questionnaire, ScrollArea, Select, Separator, Sheet, Sidebar, Skeleton, Slider, SplitPane, Tabs, Table, ToastHost, Toggle, ToggleGroup, Tooltip, TreeView, Typography, Updater | Rendering depends on shared widget colors, metrics, or assets. |
| `build()` | AspectRatio, Button, ButtonGroup, Direction, Input, Message, TextArea | All required visual data is already explicit or the widget is structural. |
| `build(theme, row/cell closure)` | List, Table, VList, SplitPane | The caller owns lazily or structurally produced children. |
| `build(trigger/target, theme)` | CommandPalette, ContextMenu, Menu | The wrapped trigger or target remains explicit, so ownership is unambiguous. |
| `build(theme, assets)` | ToastHost | Icons are an explicit application asset dependency. |
| stateful `build(&mut self, theme, context)` | AnimatedText, FilePicker | These models own retained animation or platform-task state. |

Pure state and behavior helpers (`Collection`, `CalendarState`,
`ColorPickerState`, `DataTableModel`, `DatePickerState`, `Presence`, range and
selection behaviors, `ToastState`, `TooltipState`, and `Typeahead`) intentionally
do not follow the visual builder shape because they do not independently build
a widget surface.

## Audit result

- Constructors named `new` establish stable identity and required controlled
  values; optional values remain chainable configuration.
- Theme-dependent widgets consistently accept `&WidgetTheme` at build time.
- Structural widgets do not accept a theme they cannot use.
- Collection closures remain explicit rather than hidden behind conversions.
- Direct handlers use typed payloads and are attached with `on_*` methods.
- Presentational widgets expose no invented change event. Action-only behavior
  remains available through the public action APIs.
- No deprecated constructor alias or compatibility shim was introduced. The
  repository and examples use the 0.3 signatures directly.

This inventory is the one-time 0.3 constructor cleanup. Future additions should
choose an existing row rather than add a second spelling for the same build
operation.
