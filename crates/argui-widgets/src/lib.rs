//! Optional, accessible widgets built from Argui engine primitives.
#[cfg(feature = "implicit-animation")]
mod implicit_animation;
#[cfg(feature = "implicit-animation")]
pub use implicit_animation::{AnimatedContainer, AnimatedOpacity};
#[cfg(feature = "animated-text")]
mod animated_text;
#[cfg(feature = "animated-text")]
pub use animated_text::{AnimatedText, TextAnimation};
#[cfg(feature = "color-picker")]
mod color_picker;
#[cfg(feature = "color-picker")]
pub use color_picker::{ColorFormat, ColorPicker, ColorPickerState};
#[cfg(feature = "label")]
mod label;
#[cfg(feature = "label")]
pub use label::Label;
#[cfg(feature = "skeleton")]
mod skeleton;
#[cfg(feature = "skeleton")]
pub use skeleton::Skeleton;
#[cfg(feature = "breadcrumb")]
mod breadcrumb;
#[cfg(feature = "breadcrumb")]
pub use breadcrumb::{Breadcrumb, BreadcrumbLink};
#[cfg(feature = "pagination")]
mod pagination;
#[cfg(feature = "pagination")]
pub use pagination::{Pagination, PaginationLabels};
#[cfg(feature = "menu")]
mod menu;
#[cfg(feature = "menu")]
pub use menu::{Menu, MenuIntent, MenuItem, MenuItemKind, MenuResponse};
#[cfg(feature = "command-palette")]
mod command_palette;
#[cfg(feature = "command-palette")]
pub use command_palette::CommandPalette;

#[cfg(feature = "button")]
mod button;
#[cfg(feature = "button")]
mod button_behavior;
#[cfg(feature = "dialog")]
mod dialog;
#[cfg(feature = "dialog")]
mod dialog_behavior;
#[cfg(feature = "icons")]
mod icons;
#[cfg(any(feature = "input", feature = "textarea"))]
mod input;
#[cfg(feature = "popover")]
mod popover;
#[cfg(feature = "popover")]
mod popover_behavior;
#[cfg(any(feature = "text-selection", feature = "popover", feature = "select"))]
mod presence;
#[cfg(feature = "radio-group")]
mod radio_group_behavior;
#[cfg(feature = "range")]
mod range;
#[cfg(feature = "select")]
mod select;
#[cfg(feature = "select")]
mod select_behavior;
#[cfg(any(feature = "checkbox", feature = "switch", feature = "radio-group"))]
mod selection;
#[cfg(feature = "slider")]
mod slider;
#[cfg(feature = "spinner")]
mod spinner;
#[cfg(feature = "tabs")]
mod tabs;
#[cfg(feature = "tabs")]
mod tabs_behavior;
#[cfg(any(feature = "input", feature = "textarea"))]
mod text_field_behavior;
#[cfg(feature = "text-selection")]
mod text_selection;
mod theme;
#[cfg(any(feature = "text-selection", feature = "popover", feature = "select"))]
pub use presence::Presence;
#[cfg(feature = "split-pane")]
mod split_pane;
#[cfg(feature = "split-pane")]
pub use split_pane::{SplitAxis, SplitPane};
#[cfg(feature = "vlist")]
mod vlist;
#[cfg(feature = "vlist")]
pub use vlist::VList;
#[cfg(feature = "tree-view")]
mod tree_view;
#[cfg(feature = "tree-view")]
pub use tree_view::{TreeAction, TreeNode, TreeView, TreeViewCache};
#[cfg(any(feature = "checkbox", feature = "switch", feature = "radio-group"))]
mod toggle_behavior;

#[cfg(feature = "button")]
pub use button::{Button, ButtonStyle};
#[cfg(feature = "button")]
pub use button_behavior::{BUTTON_BUSY, BUTTON_SCOPE, ButtonAction, ButtonBehavior, ButtonPart};
#[cfg(feature = "dialog")]
pub use dialog::{Dialog, DialogPlacement};
#[cfg(feature = "dialog")]
pub use dialog_behavior::{DIALOG_OPEN, DIALOG_SCOPE, DialogAction, DialogBehavior, DialogPart};
#[cfg(feature = "icons")]
pub use icons::{TablerIcon, WidgetAssets};
#[cfg(any(feature = "input", feature = "textarea"))]
pub use input::InputStyle;
#[cfg(feature = "textarea")]
pub use input::TextArea;
#[cfg(feature = "input")]
pub use input::{Input, InputKind};
#[cfg(feature = "popover")]
pub use popover::Popover;
#[cfg(feature = "popover")]
pub use popover_behavior::{
    POPOVER_OPEN, POPOVER_SCOPE, PopoverAction, PopoverBehavior, PopoverPart,
};
#[cfg(feature = "radio-group")]
pub use radio_group_behavior::{RadioGroupAction, RadioGroupBehavior, RadioGroupPart};
#[cfg(feature = "range")]
pub use range::{
    RANGE_SCOPE, RangeAction, RangeAxis, RangeBehavior, RangeConfig, RangeDetents, RangeDirection,
    RangePart, RangeState,
};
#[cfg(feature = "select")]
pub use select::{Select, SelectOption};
#[cfg(feature = "select")]
pub use select_behavior::{
    SELECT_HIGHLIGHTED, SELECT_OPEN, SELECT_SCOPE, SELECT_SELECTED, SelectAction, SelectBehavior,
    SelectPart,
};
#[cfg(feature = "checkbox")]
pub use selection::Checkbox;
#[cfg(feature = "switch")]
pub use selection::Switch;
#[cfg(feature = "radio-group")]
pub use selection::{RadioGroup, RadioOption};
#[cfg(feature = "slider")]
pub use slider::Slider;
#[cfg(feature = "spinner")]
pub use spinner::Spinner;
#[cfg(feature = "tabs")]
pub use tabs::{Tab, Tabs};
#[cfg(feature = "tabs")]
pub use tabs_behavior::{TAB_SELECTED, TABS_SCOPE, TabsAction, TabsBehavior, TabsPart};
#[cfg(any(feature = "input", feature = "textarea"))]
pub use text_field_behavior::{
    TEXT_FIELD_INVALID, TEXT_FIELD_READ_ONLY, TEXT_FIELD_SCOPE, TextFieldBehavior, TextFieldPart,
};
#[cfg(feature = "text-selection")]
pub use text_selection::{SelectionHost, TextSelectionToolbar};
pub use theme::{WidgetTheme, default_theme, shadcn};
#[cfg(any(feature = "checkbox", feature = "switch", feature = "radio-group"))]
pub use toggle_behavior::{TOGGLE_CHECKED, TOGGLE_SCOPE, ToggleAction, ToggleBehavior, TogglePart};

#[cfg(feature = "list")]
mod collection;
#[cfg(feature = "list")]
mod list;
#[cfg(feature = "list")]
pub use collection::{Collection, CollectionItem, DuplicateItemId, ListState};
#[cfg(feature = "list")]
pub use list::List;
#[cfg(feature = "table")]
mod table;
#[cfg(feature = "table")]
pub use table::{Table, TableColumn};

mod typeahead;
pub use typeahead::{Typeahead, TypeaheadConfig, unicode_prefix};

#[cfg(feature = "calendar")]
mod calendar;
#[cfg(feature = "calendar")]
pub use calendar::{
    Calendar, CalendarConstraints, CalendarLocale, CalendarSelection, CalendarState,
    IsoCalendarLocale,
};
#[cfg(feature = "calendar")]
pub use time::{Date, Month, Weekday};

#[cfg(feature = "date-picker")]
mod date_picker;
#[cfg(feature = "date-picker")]
pub use date_picker::{DatePicker, DatePickerResponse, DatePickerState};

#[cfg(feature = "toast")]
mod toast;
#[cfg(feature = "toast")]
pub use toast::{Toast, ToastHost, ToastInsertError, ToastPause, ToastState, ToastVariant};

#[cfg(feature = "data-table")]
mod data_table;
#[cfg(feature = "data-table")]
pub use data_table::{
    CellAddress, CellCommit, CellEdit, DataColumn, DataPage, DataRow, DataSort, DataTable,
    DataTableAction, DataTableError, DataTableModel,
};

#[cfg(feature = "context-menu")]
mod context_menu;
#[cfg(feature = "context-menu")]
pub use context_menu::ContextMenu;

#[cfg(feature = "menubar")]
mod menubar;
#[cfg(feature = "menubar")]
pub use menubar::{Menubar, MenubarResponse};

#[cfg(feature = "badge")]
mod badge;
#[cfg(feature = "badge")]
pub use badge::{Badge, BadgeVariant};

#[cfg(feature = "card")]
mod card;
#[cfg(feature = "card")]
pub use card::Card;

#[cfg(feature = "alert")]
mod alert;
#[cfg(feature = "alert")]
pub use alert::{Alert, AlertVariant};

#[cfg(feature = "separator")]
mod separator;
#[cfg(feature = "separator")]
pub use separator::Separator;

#[cfg(feature = "collapsible")]
mod collapsible;
#[cfg(feature = "collapsible")]
pub use collapsible::Collapsible;

#[cfg(feature = "avatar")]
mod avatar;
#[cfg(feature = "avatar")]
pub use avatar::Avatar;

#[cfg(feature = "empty")]
mod empty;
#[cfg(feature = "empty")]
pub use empty::Empty;

#[cfg(feature = "kbd")]
mod kbd;
#[cfg(feature = "kbd")]
pub use kbd::Kbd;

#[cfg(feature = "aspect-ratio")]
mod aspect_ratio;
#[cfg(feature = "aspect-ratio")]
pub use aspect_ratio::AspectRatio;

#[cfg(feature = "progress")]
mod progress;
#[cfg(feature = "progress")]
pub use progress::Progress;

#[cfg(feature = "tooltip")]
mod tooltip;
#[cfg(feature = "tooltip")]
pub use tooltip::{Tooltip, TooltipHost, TooltipState};

#[cfg(feature = "file-picker")]
mod file_picker;
#[cfg(feature = "file-picker")]
pub use file_picker::{FilePicker, FilePickerEvent, FilePickerStatus};

#[cfg(any(
    feature = "accordion",
    feature = "toggle-group",
    feature = "combobox",
    feature = "navigation-menu",
    feature = "questionnaire"
))]
mod choice_navigation;
#[cfg(any(
    feature = "accordion",
    feature = "toggle-group",
    feature = "combobox",
    feature = "navigation-menu",
    feature = "questionnaire"
))]
pub use choice_navigation::ChoiceMode;

#[cfg(feature = "toggle")]
mod toggle;
#[cfg(feature = "toggle")]
pub use toggle::Toggle;

#[cfg(feature = "toggle-group")]
mod toggle_group;
#[cfg(feature = "toggle-group")]
pub use toggle_group::{ToggleGroup, ToggleGroupAction};

#[cfg(feature = "accordion")]
mod accordion;
#[cfg(feature = "accordion")]
pub use accordion::{Accordion, AccordionAction, AccordionItem};

#[cfg(feature = "button-group")]
mod button_group;
#[cfg(feature = "button-group")]
pub use button_group::ButtonGroup;

#[cfg(feature = "item")]
mod item;
#[cfg(feature = "item")]
pub use item::Item;

#[cfg(feature = "field")]
mod field;
#[cfg(feature = "field")]
pub use field::Field;

#[cfg(feature = "input-group")]
mod input_group;
#[cfg(feature = "input-group")]
pub use input_group::InputGroup;

#[cfg(feature = "scroll-area")]
mod scroll_area;
#[cfg(feature = "scroll-area")]
pub use scroll_area::ScrollArea;

#[cfg(feature = "typography")]
mod typography;
#[cfg(feature = "typography")]
pub use typography::{Typography, TypographyVariant};

#[cfg(feature = "sheet")]
mod sheet;
#[cfg(feature = "sheet")]
pub use sheet::{Sheet, SheetSide};

#[cfg(feature = "alert-dialog")]
mod alert_dialog;
#[cfg(feature = "alert-dialog")]
pub use alert_dialog::{AlertDialog, AlertDialogAction};

#[cfg(feature = "combobox")]
mod combobox;
#[cfg(feature = "combobox")]
pub use combobox::{Combobox, ComboboxAction};

#[cfg(feature = "native-select")]
mod native_select;
#[cfg(feature = "native-select")]
pub use native_select::NativeSelect;

#[cfg(feature = "input-otp")]
mod input_otp;
#[cfg(feature = "input-otp")]
pub use input_otp::InputOtp;

#[cfg(feature = "direction")]
mod direction;
#[cfg(feature = "direction")]
pub use direction::Direction;

#[cfg(feature = "bubble")]
mod bubble;
#[cfg(feature = "bubble")]
pub use bubble::{Bubble, BubbleVariant};

#[cfg(feature = "message")]
mod message;
#[cfg(feature = "message")]
pub use message::Message;

#[cfg(feature = "marker")]
mod marker;
#[cfg(feature = "marker")]
pub use marker::{Marker, MarkerVariant};

#[cfg(feature = "attachment")]
mod attachment;
#[cfg(feature = "attachment")]
pub use attachment::{Attachment, AttachmentState};

#[cfg(feature = "message-scroller")]
mod message_scroller;
#[cfg(feature = "message-scroller")]
pub use message_scroller::{MessageScrollState, MessageScroller, MessageScrollerAction};

#[cfg(feature = "hover-card")]
mod hover_card;
#[cfg(feature = "hover-card")]
pub use hover_card::{HoverCard, HoverCardState};

#[cfg(feature = "navigation-menu")]
mod navigation_menu;
#[cfg(feature = "navigation-menu")]
pub use navigation_menu::{NavigationItem, NavigationMenu, NavigationMenuAction};

#[cfg(feature = "sidebar")]
mod sidebar;
#[cfg(feature = "sidebar")]
pub use sidebar::{Sidebar, SidebarAction};

#[cfg(feature = "drawer")]
mod drawer;
#[cfg(feature = "drawer")]
pub use drawer::{Drawer, DrawerAction};

#[cfg(feature = "carousel")]
mod carousel;
#[cfg(feature = "carousel")]
pub use carousel::Carousel;

#[cfg(feature = "chart")]
mod chart;
#[cfg(feature = "chart")]
pub use chart::{Chart, ChartError, ChartKind, ChartSeries};

#[cfg(feature = "questionnaire")]
mod questionnaire;
#[cfg(feature = "questionnaire")]
pub use questionnaire::{
    Question, QuestionAnswer, QuestionChoice, Questionnaire, QuestionnaireAction,
    QuestionnaireState,
};

#[cfg(feature = "updater")]
mod updater;
#[cfg(feature = "updater")]
pub use updater::{UpdateAction, UpdateDialog};
