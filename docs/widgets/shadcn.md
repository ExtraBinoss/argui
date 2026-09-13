# Widget catalogue

Browse the [live components](https://extrabinoss.github.io/argui/components) or
open an implementation below. The catalogue maps familiar shadcn/ui component
names to Argui's Rust APIs and describes their supported scope. It does not
promise parity with every React variant or replace real screen-reader testing.

## APIs and features

Enable the listed feature in [argui-widgets](../../crates/argui-widgets/Cargo.toml),
or prefix it with `widget-` in [argui](../../crates/argui/Cargo.toml).
`all` / `widgets-all` enables this collection. Defaults are empty, and the
updater dialog is a separate opt-in.

| Component | Public API | Feature | Supported behavior and limits |
| --- | --- | --- | --- |
| Accordion | [Accordion](../../crates/argui-widgets/src/accordion.rs) | `accordion` | Controlled single/multiple sections, arrow/Home/End navigation and disabled states; opening is not animated. |
| Alert | [Alert](../../crates/argui-widgets/src/alert.rs) | `alert` | Inline standard/destructive presentation, icon and explicit announcement policy. |
| Alert Dialog | [AlertDialog](../../crates/argui-widgets/src/alert_dialog.rs) | `alert-dialog` | Modal confirmation, initial Cancel focus, ignored outside clicks and a distinct confirmation action. |
| Aspect Ratio | [AspectRatio](../../crates/argui-widgets/src/aspect_ratio.rs) | `aspect-ratio` | Reserves height from width and a positive aspect ratio; fits content to the frame. |
| Attachment | [Attachment](../../crates/argui-widgets/src/attachment.rs) | `attachment` | File metadata, media, actions, five transfer states and progress; the application owns the transfer. |
| Avatar | [Avatar](../../crates/argui-widgets/src/avatar.rs) | `avatar` | Loaded image or controlled fallback, circular clip, size and a single accessible name; no avatar group. |
| Badge | [Badge](../../crates/argui-widgets/src/badge.rs) | `badge` | Primary/secondary/destructive/outline/ghost variants, leading/trailing icons and an accessible name. |
| Breadcrumb | [Breadcrumb](../../crates/argui-widgets/src/breadcrumb.rs) | `breadcrumb` | Actionable ancestors, stable IDs, custom separators and a described current page; accessible group without a Navigation landmark. |
| Bubble | [Bubble](../../crates/argui-widgets/src/bubble.rs) | `bubble` | Seven variants, alignment and reactions; content is arbitrary and must use suitable foreground colors. |
| Button | [Button](../../crates/argui-widgets/src/button.rs) | `button` | Theme variants, icons, loading state and accessible activation. |
| Button Group | [ButtonGroup](../../crates/argui-widgets/src/button_group.rs) | `button-group` | Named horizontal/vertical group; each button keeps its own Tab stop. |
| Calendar | [Calendar](../../crates/argui-widgets/src/calendar.rs) | `calendar` | Calendar and retained CalendarState, locale, bounds, single/multiple/range selection and keyboard navigation. |
| Card | [Card](../../crates/argui-widgets/src/card.rs) | `card` | Optional title, description, actions, content and footer with accessible relationships. |
| Carousel | [Carousel](../../crates/argui-widgets/src/carousel.rs) | `carousel` | Controlled slide, buttons, keyboard, swiping, optional looping and announcements; no autoplay. |
| Chart | [Chart](../../crates/argui-widgets/src/chart.rs) | `chart` | Multi-series bars and lines, a zero-inclusive scale, legend and actionable accessible points; other plots are not provided. |
| Checkbox | [Checkbox](../../crates/argui-widgets/src/selection.rs) | `checkbox` | CheckedState supports Unchecked, Checked and Mixed; activating Mixed moves to Checked. |
| Collapsible | [Collapsible](../../crates/argui-widgets/src/collapsible.rs) | `collapsible` | Controlled opening, custom trigger, Enter/Space, disabled state and unmounted closed content. |
| Combobox | [Combobox](../../crates/argui-widgets/src/combobox.rs) | `combobox` | Editable search, filtering, disabled options, active descendant and keyboard selection; single selection. |
| Command | [CommandPalette](../../crates/argui-widgets/src/command_palette.rs) | `command-palette` | Action search and invocation; advanced grouping and variants remain limited. |
| Context Menu | [ContextMenu](../../crates/argui-widgets/src/context_menu.rs) | `context-menu` | Shared Menu entries with pointer or keyboard anchoring. |
| Data Table | [DataTable](../../crates/argui-widgets/src/data_table.rs) | `data-table` | Typed model, filters, stable multi-column sorting, pagination, visible columns, virtualization and controlled editing; headers track scrolling. |
| Date Picker | [DatePicker](../../crates/argui-widgets/src/date_picker.rs) | `date-picker` | Localizable input, controlled draft, validation and a reused calendar. |
| Dialog | [Dialog](../../crates/argui-widgets/src/dialog.rs) | `dialog` | Modal focus trapping/restoration, configurable initial focus, placement and scrollable content. |
| Direction | [Direction](../../crates/argui-widgets/src/direction.rs) | `direction` | Inherited layout direction with nested scopes; keyboard controllers take an explicit rtl setting. |
| Drawer | [Drawer](../../crates/argui-widgets/src/drawer.rs) | `drawer` | Modal bottom panel, drag handle, distance/velocity thresholds and cancellation; no intermediate snap positions. |
| Dropdown Menu | [Menu](../../crates/argui-widgets/src/menu.rs) | `menu` | Stable IDs, submenus, groups, checkbox/radio entries, indicators and keyboard navigation. |
| Empty | [Empty](../../crates/argui-widgets/src/empty.rs) | `empty` | Centered empty state, title, description, decorative media and arbitrary actions; optional border. |
| Field | [Field](../../crates/argui-widgets/src/field.rs) | `field` | Label, help, error, required and disabled states linked to a control key; the application owns validation. |
| Hover Card | [HoverCard](../../crates/argui-widgets/src/hover_card.rs) | `hover-card` | Interactive nonmodal preview, delays, pointer movement between trigger/content, focus and dismissal. |
| Input | [Input](../../crates/argui-widgets/src/input.rs) | `input` | Controlled text, search and password input. |
| Input Group | [InputGroup](../../crates/argui-widgets/src/input_group.rs) | `input-group` | Shared surface with an editor, decorations and leading/trailing actions. |
| Input OTP | [InputOtp](../../crates/argui-widgets/src/input_otp.rs) | `input-otp` | One editor for 1–16 spaced ASCII digits, filtering before mutation and completion; no separate inputs or SMS retrieval. |
| Item | [Item](../../crates/argui-widgets/src/item.rs) | `item` | Reusable row with title, description, media and actions. |
| Kbd | [Kbd](../../crates/argui-widgets/src/kbd.rs) | `kbd` | Key or key combination with a customizable announcement; does not register shortcuts. |
| Label | [Label](../../crates/argui-widgets/src/label.rs) | `label` | Visible label linked through labelled_by, click-to-focus and disabled state; no extra Tab stop. |
| Marker | [Marker](../../crates/argui-widgets/src/marker.rs) | `marker` | Inline, bordered or divider note with an optional icon and announcement. |
| Menubar | [Menubar](../../crates/argui-widgets/src/menubar.rs) | `menubar` | Trigger focus, menus, submenus and RTL keyboard navigation. |
| Message | [Message](../../crates/argui-widgets/src/message.rs) | `message` | Author, avatar, header, content and footer with configurable alignment. |
| Message Scroller | [MessageScroller](../../crates/argui-widgets/src/message_scroller.rs) | `message-scroller` | Follow-bottom behavior, pause while reading, unread count, jump-to-latest and preserved offset when prepending. |
| Native Select | [NativeSelect](../../crates/argui-widgets/src/native_select.rs) | `native-select` | Compact Argui-rendered selector with required/disabled states, keyboard navigation and search; not an OS control. |
| Navigation Menu | [NavigationMenu](../../crates/argui-widgets/src/navigation_menu.rs) | `navigation-menu` | Navigation landmark, actions, panels, current page and keyboard navigation. |
| Pagination | [Pagination](../../crates/argui-widgets/src/pagination.rs) | `pagination` | Controlled navigation, bounds, ellipses, disabled state and translatable announcements; accessible group with a described current page. |
| Popover | [Popover](../../crates/argui-widgets/src/popover.rs) | `popover` | Anchoring, collision handling, dismissal and focus options. |
| Progress | [Progress](../../crates/argui-widgets/src/progress.rs) | `progress` | Controlled percentage, indeterminate animation when mounted as an Entity, reduced motion and accessible value. |
| Questionnaire | [Questionnaire](../../crates/argui-widgets/src/questionnaire.rs) | `questionnaire` | Steps, single/multiple choice, free text, optional answers and validation; returns answers to the application. |
| Radio Group | [RadioGroup](../../crates/argui-widgets/src/selection.rs) | `radio-group` | Exclusive selection and configurable orientation. |
| Resizable | [SplitPane](../../crates/argui-widgets/src/split_pane.rs) | `split-pane` | Divider, limits, gestures and keyboard resizing; compose nested groups explicitly. |
| Scroll Area | [ScrollArea](../../crates/argui-widgets/src/scroll_area.rs) | `scroll-area` | Horizontal/vertical viewport, wheel and gesture input, scrollbars and focused keyboard scrolling; explicit layout boundary for internal changes. |
| Select | [Select](../../crates/argui-widgets/src/select.rs) | `select` | Controlled single selection, overlay and keyboard navigation. |
| Separator | [Separator](../../crates/argui-widgets/src/separator.rs) | `separator` | Configurable orientation and centered label; decorative by default, with an optional accessible role. |
| Sheet | [Sheet](../../crates/argui-widgets/src/sheet.rs) | `sheet` | Modal panel on any of the four edges, focus restoration and scrollable content. |
| Sidebar | [Sidebar](../../crates/argui-widgets/src/sidebar.rs) | `sidebar` | Expanded navigation, collapsed rail or a mobile Sheet; the application chooses the breakpoint. |
| Skeleton | [Skeleton](../../crates/argui-widgets/src/skeleton.rs) | `skeleton` | Sized decorative shapes, mount-owned pulse animation and reduced motion. |
| Slider | [Slider](../../crates/argui-widgets/src/slider.rs) | `slider` | Value, bounds, step, gestures and keyboard control; multiple thumbs are not provided. |
| Spinner | [Spinner](../../crates/argui-widgets/src/spinner.rs) | `spinner` | Mount-owned animation and reduced motion. |
| Switch | [Switch](../../crates/argui-widgets/src/selection.rs) | `switch` | Animated boolean control with accessible activation. |
| Table | [Table](../../crates/argui-widgets/src/table.rs) | `table` | Headers, cells, shared column widths, row selection and navigation; use DataTable for advanced data operations. |
| Tabs | [Tabs](../../crates/argui-widgets/src/tabs.rs) | `tabs` | Selection, navigation and mounting of the active panel. |
| Textarea | [TextArea](../../crates/argui-widgets/src/input.rs) | `textarea` | Multiline editing and scrolling; compose resizing with public gestures. |
| Toast | [Toast](../../crates/argui-widgets/src/toast.rs) | `toast` | Controlled queue, duration, hover/focus pause, announcements and actions. |
| Toggle | [Toggle](../../crates/argui-widgets/src/toggle.rs) | `toggle` | Persistent pressed state, disabled state and visual variants. |
| Toggle Group | [ToggleGroup](../../crates/argui-widgets/src/toggle_group.rs) | `toggle-group` | Single/multiple selection, orientation, RTL and roving focus. |
| Tooltip | [Tooltip](../../crates/argui-widgets/src/tooltip.rs) | `tooltip` | Delays, hover/focus, descriptive relationships, hoverable content, effects and optional native presentation. |
| Typography | [Typography](../../crates/argui-widgets/src/typography.rs) | `typography` | Headings h1–h6, paragraph, lead, large, small, muted, code and quote styles; the engine also supports rich text. |

## Integration

Build the view with its theme, forward events to `action`, apply the returned
action to retained state and notify the owning entity. Forward focus requests
to the runtime. Widget models do not perform network transfers or submit answers.

Complete examples:

- [Forms](../../crates/argui-widget-gallery/src/pages/catalogue/forms.rs): Field,
  Input Group, OTP, Combobox, Native Select and Questionnaire.
- [Navigation](../../crates/argui-widget-gallery/src/pages/catalogue/navigation.rs):
  Accordion, toggles, Button Group, Navigation Menu, Sidebar and Direction.
- [Surfaces](../../crates/argui-widget-gallery/src/pages/catalogue/surfaces.rs):
  Sheet, Alert Dialog, Drawer, Carousel, Chart and Hover Card.
- [Conversation](../../crates/argui-widget-gallery/src/pages/catalogue/conversation.rs):
  Attachment, Bubble, Item, Marker, Message, Message Scroller and Scroll Area.
- [Typography](../../crates/argui-widget-gallery/src/pages/typography.rs): text styles,
  rich text and selection.

Keep `HoverCardState`, capture Focus/Blur/Key on the panel and schedule `advance`
at `next_deadline` through a `TaskSlot`. For MessageScroller, provide layout
measurements to `appended`/`prepended`, pass reading events to `observe` and
apply returned `ScrollRequest`s. Virtualization and network streaming are
application responsibilities.

Direction inherits through layout and supports opposing nested scopes. Set
collection controllers' `rtl` parameter to the same direction for arrow-key
interpretation. The engine stays independent of widgets and any future DSL.

## Composition contracts

### Content and disclosure

Card and Empty connect title/description to their accessible group while their
actions retain focus. Badge is noninteractive. Decorative media should not
produce duplicate announcements. Use `Alert::live(LiveRegion::Off)` for an
already-present persistent alert that should not announce dynamically.

Collapsible unmounts closed content. `trigger_key()` and `content_key()` identify
its parts. The application retains data and restores focus when closing a
section containing the focused control. Height changes are not animated.

Separator is horizontal by default; a vertical separator needs a parent with
a defined height. `label` centers text. `decorative(false)` exposes its role
and orientation without announcing children twice.

### Images and loading

`Avatar::image` receives an already-loaded `Option<ImageId>`; the application
chooses the image or fallback. AspectRatio requires a finite positive ratio;
clip its frame when needed. Kbd describes keys without registering shortcuts;
`label` supplies a spoken name for symbols.

Mount `Entity<Progress>` or `Entity<Skeleton>` with `cx.entity` for animation.
Calling `build` directly is static. `Progress::set_value` takes a percentage
clamped to 0–100; `None` or a non-finite value selects indeterminate progress.
Reduced motion, unmounting and `Skeleton::set_animated(false)` stop frame requests.
Announce loading on the parent group instead of every decorative shape.

### Labels and navigation

`Label::associate` sets the control key and replaces its direct name with
`labelled_by`. Keep both in one accessible scope with matching enabled states,
and forward `focus_target(event)` to the runtime. Clicking focuses without
changing the control value.

`Breadcrumb::action` returns a unique ancestor ID for application navigation.
`Pagination::action` returns an available destination. Pages start at 1; an
empty total produces page 0 and disabled buttons. At most seven entries appear
between previous/next. `PaginationLabels` provides translated announcements.
Both widgets currently expose a named group and current-page description,
without `aria-current`.

`Menu::submenu_content_key` identifies the geometry anchor for MenuIntent.
Supply bounds after layout, update the pointer-corridor deadline during movement
and cancel its task when closing. `CalendarLocale::rtl` controls horizontal
navigation. DatePicker scopes events and restores the committed date on cancel.

## Additional widgets

**AnimatedText** (`animated-text`) animates changed characters through rolling,
sliding or fading, reuses layout between intermediate frames and stops requesting
frames at rest. See [animation](../ui/animation.md).

**ColorPicker** (`color-picker`) includes saturation/value, hue, checkerboard
opacity and HEX/RGB/HSL/HSV fields. `ColorPickerState` retains hue through gray,
captured gestures and invalid drafts without replacing the last valid color.
Call `layout_changed(key, snapshot)` after layout, then `update(key, event)`;
`true` requests a rebuild. Compare `color()` before/after to publish only changes.
Use `set_color`, `set_format` and `set_enabled` for external state. Arrow keys
adjust sliders and the pad; Shift refines pad steps to 0.1%, and Escape discards
an invalid draft.

See dedicated guides for [lists and tables](lists-tables.md),
[overlays](overlays.md), [native popovers](../platform/native-popovers.md),
[file pickers](../platform/file-picker.md), [WebViews](../platform/webview.md),
[desktop backdrops](../platform/desktop-backdrops.md) and
[application updates](../platform/updater.md).

## Validation

[Widget tests](../../crates/argui-widgets/tests/) cover actions, limits, disabled
states, accessible relationships, gestures and layout. The
[gallery tests](../../crates/argui-widget-gallery/tests/pages/catalogue.rs) exercise
retained states in light/dark themes and at two viewport sizes.

The [browser scenario](../../crates/argui-widget-gallery/tests/pages/catalogue.mjs)
covers actual WebGPU output, navigation, input, modals, focus restoration,
Hover Card and scrolling. Inspect its captures in `target/catalogue-interactions/`;
a blank image fails. Use the [private Linux display](../contributing/linux-testing.md)
and the [quality gate](../contributing/code-quality.md).
