use super::*;
use crate::app::text;
use argui::{
    ui::{ActionId, ActionInvocation, ActionState, Element, Orientation, WritingDirection, length},
    widgets::*,
};

const FOLLOW_MENU: &str = "group-follow-menu";
const CURRENCY_SELECT: &str = "group-currency";
const COPILOT_POPOVER: &str = "group-copilot-menu";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Identifies the mutually exclusive overlay shown by the Button Group demo.
pub(super) enum ButtonGroupOverlay {
    Menu,
    Select,
    Popover,
}

/// Retained values used by the interactive Button Group examples.
#[derive(Debug)]
pub(super) struct ButtonGroupState {
    pub(super) search: String,
    pub(super) nested_message: String,
    pub(super) message: String,
    pub(super) amount: String,
    pub(super) task: String,
    pub(super) currency: usize,
    pub(super) highlighted_currency: usize,
    pub(super) voice: bool,
    pub(super) overlay: Option<ButtonGroupOverlay>,
}

impl Default for ButtonGroupState {
    fn default() -> Self {
        Self {
            search: String::new(),
            nested_message: String::new(),
            message: String::new(),
            amount: "10.00".into(),
            task: String::new(),
            currency: 0,
            highlighted_currency: 0,
            voice: false,
            overlay: None,
        }
    }
}

/// Wraps one Button Group example with its section title.
fn section(title: &str, content: Element, theme: &WidgetTheme) -> Element {
    Element::column([text(title, 13.0, theme.muted_foreground, 600), content]).gap(10.0)
}

/// Returns a button style with the requested demo dimensions.
fn sized(mut style: ButtonStyle, height: f32, padding: f32, font_size: f32) -> ButtonStyle {
    style.layout.size.height = length(height);
    style.layout.padding = argui::ui::sides(padding, 0.0);
    style.label.font_size = font_size;
    style
}

/// Builds a text button whose key is namespaced for the Button Group page.
fn button(key: &str, label: &str, style: ButtonStyle) -> Element {
    Button::new(format!("group-{key}"), label, style).build()
}

/// Builds an accessible symbol button for a compact grouped action.
fn symbol_button(
    key: &str,
    label: &str,
    symbol: &str,
    style: ButtonStyle,
    theme: &WidgetTheme,
) -> Element {
    Button::new(format!("group-{key}"), label, style)
        .content(text(symbol, 17.0, theme.foreground, 500))
        .build()
}

/// Returns the currencies used by both the Select view and its behavior.
pub(super) fn currency_options() -> [SelectOption; 3] {
    ["$", "€", "£"].map(SelectOption::new)
}

impl CatalogueDemo {
    /// Creates the controlled dropdown menu used by the split Follow button.
    pub(super) fn button_group_menu(&self) -> Menu {
        Menu::new(
            FOLLOW_MENU,
            "Follow options",
            self.button_group.overlay == Some(ButtonGroupOverlay::Menu),
            [
                ("all", "All notifications", "gallery.follow.all"),
                ("mentions", "Mentions only", "gallery.follow.mentions"),
                ("unfollow", "Unfollow", "gallery.follow.none"),
            ]
            .map(|(id, label, action)| {
                MenuItem::new(
                    id,
                    ActionInvocation::new(ActionId(action)),
                    ActionState::new(label),
                )
            })
            .into(),
        )
    }

    /// Creates interaction behavior matching the controlled currency Select.
    pub(super) fn button_group_select_behavior(&self) -> SelectBehavior {
        SelectBehavior::new(
            CURRENCY_SELECT,
            "Currency",
            currency_options(),
            Some(self.button_group.currency),
        )
        .open(self.button_group.overlay == Some(ButtonGroupOverlay::Select))
        .highlighted(self.button_group.highlighted_currency)
    }

    /// Creates interaction behavior matching the controlled Copilot popover.
    pub(super) fn button_group_popover_behavior(&self) -> PopoverBehavior {
        PopoverBehavior::new(
            COPILOT_POPOVER,
            "Open Copilot",
            self.button_group.overlay == Some(ButtonGroupOverlay::Popover),
        )
    }

    /// Builds the complete interactive Button Group catalogue page.
    pub(super) fn button_group_view(&self, theme: &WidgetTheme) -> Element {
        let outline = || theme.outline_button();
        let overview = ButtonGroup::new(
            "mail-actions",
            "Mail actions",
            [
                ButtonGroup::new(
                    "mail-back",
                    "Back",
                    [symbol_button("back", "Go back", "<", outline(), theme)],
                )
                .build(),
                ButtonGroup::new(
                    "mail-primary",
                    "Message actions",
                    [
                        button("archive", "Archive", outline()),
                        button("report", "Report", outline()),
                    ],
                )
                .build(),
                ButtonGroup::new(
                    "mail-more",
                    "Snooze options",
                    [
                        button("snooze", "Snooze", outline()),
                        symbol_button("more", "More options", "•••", outline(), theme),
                    ],
                )
                .build(),
            ],
        )
        .spacing(8.0)
        .build();

        let orientation = ButtonGroup::new(
            "media-controls",
            "Media controls",
            [
                symbol_button("increase", "Increase", "+", outline(), theme),
                symbol_button("decrease", "Decrease", "−", outline(), theme),
            ],
        )
        .orientation(Orientation::Vertical)
        .build();

        let sizes = Element::column([
            ButtonGroup::new(
                "small-actions",
                "Small actions",
                [
                    button("small", "Small", sized(outline(), 32.0, 12.0, 13.0)),
                    button("small-button", "Button", sized(outline(), 32.0, 12.0, 13.0)),
                    button("small-group", "Group", sized(outline(), 32.0, 12.0, 13.0)),
                    symbol_button(
                        "small-add",
                        "Add",
                        "+",
                        sized(outline(), 32.0, 10.0, 13.0),
                        theme,
                    ),
                ],
            )
            .build(),
            ButtonGroup::new(
                "default-actions",
                "Default actions",
                [
                    button("default", "Default", outline()),
                    button("default-button", "Button", outline()),
                    button("default-group", "Group", outline()),
                    symbol_button("default-add", "Add", "+", outline(), theme),
                ],
            )
            .build(),
            ButtonGroup::new(
                "large-actions",
                "Large actions",
                [
                    button("large", "Large", sized(outline(), 40.0, 18.0, 15.0)),
                    button("large-button", "Button", sized(outline(), 40.0, 18.0, 15.0)),
                    button("large-group", "Group", sized(outline(), 40.0, 18.0, 15.0)),
                    symbol_button(
                        "large-add",
                        "Add",
                        "+",
                        sized(outline(), 40.0, 13.0, 15.0),
                        theme,
                    ),
                ],
            )
            .build(),
        ])
        .gap(12.0);

        let composer = InputGroup::new(
            "group-composer",
            "Message",
            Input::new(
                "group-composer-input",
                &self.button_group.nested_message,
                "Send a message...",
                theme.input(),
            )
            .label("Message")
            .build(),
        )
        .build(theme)
        .width(length(320.0));
        let nested = ButtonGroup::new(
            "nested-actions",
            "Compose message",
            [
                ButtonGroup::new(
                    "attachment-actions",
                    "Attachments",
                    [symbol_button(
                        "attachment",
                        "Add attachment",
                        "+",
                        outline(),
                        theme,
                    )],
                )
                .build(),
                ButtonGroup::new("message-input", "Message input", [composer]).build(),
            ],
        )
        .spacing(8.0)
        .build();

        let separated = ButtonGroup::new(
            "clipboard-actions",
            "Clipboard actions",
            [
                button("copy", "Copy", theme.secondary_button()),
                ButtonGroupSeparator::new("clipboard-separator").build(theme),
                button("paste", "Paste", theme.secondary_button()),
            ],
        )
        .build();
        let split = ButtonGroup::new(
            "split-actions",
            "Split action",
            [
                button("split", "Button", theme.secondary_button()),
                ButtonGroupSeparator::new("split-separator").build(theme),
                symbol_button("split-add", "Add", "+", theme.secondary_button(), theme),
            ],
        )
        .build();

        let search = Input::new(
            "group-search",
            &self.button_group.search,
            "Search...",
            theme.input(),
        )
        .label("Search")
        .build()
        .grow(1.0)
        .shrink(1.0)
        .min_width(length(0.0));
        let input = ButtonGroup::new(
            "search-actions",
            "Search",
            [
                search,
                symbol_button("search-submit", "Search", "Go", outline(), theme),
            ],
        )
        .build()
        .width(length(340.0));

        let mut voice_input = InputGroup::new(
            "voice-message",
            "Voice message",
            Input::new(
                "voice-message-input",
                &self.button_group.message,
                if self.button_group.voice {
                    "Record and send audio..."
                } else {
                    "Send a message..."
                },
                theme.input(),
            )
            .label("Message")
            .enabled(!self.button_group.voice)
            .build(),
        );
        voice_input.trailing = Some(symbol_button(
            "voice",
            "Toggle voice mode",
            if self.button_group.voice {
                "Stop"
            } else {
                "Mic"
            },
            theme.ghost_button(),
            theme,
        ));
        let input_group = ButtonGroup::new(
            "composer-actions",
            "Composer",
            [
                ButtonGroup::new(
                    "composer-add",
                    "Add",
                    [symbol_button("composer-add", "Add", "+", outline(), theme)],
                )
                .build(),
                ButtonGroup::new(
                    "composer-field",
                    "Message",
                    [voice_input.build(theme).width(length(320.0))],
                )
                .build(),
            ],
        )
        .spacing(8.0)
        .build();

        let dropdown_menu = self.button_group_menu().build(
            symbol_button("follow-menu", "Follow options", "v", outline(), theme),
            theme,
        );
        let dropdown = ButtonGroup::new(
            "follow-actions",
            "Follow options",
            [button("follow", "Follow", outline()), dropdown_menu],
        )
        .build();

        let amount = Input::new(
            "group-amount",
            &self.button_group.amount,
            "10.00",
            theme.input(),
        )
        .label("Amount")
        .build()
        .width(length(140.0));
        let currency = Select::new(
            CURRENCY_SELECT,
            "Currency",
            currency_options(),
            Some(self.button_group.currency),
        )
        .open(self.button_group.overlay == Some(ButtonGroupOverlay::Select))
        .highlighted(self.button_group.highlighted_currency)
        .trailing(text("v", 13.0, theme.muted_foreground, 600))
        .build(theme)
        .width(length(64.0));
        let select = ButtonGroup::new(
            "currency-actions",
            "Currency amount",
            [
                ButtonGroup::new("currency-input", "Currency and amount", [currency, amount])
                    .build(),
                ButtonGroup::new(
                    "currency-send",
                    "Send amount",
                    [symbol_button(
                        "send-amount",
                        "Send amount",
                        ">",
                        outline(),
                        theme,
                    )],
                )
                .build(),
            ],
        )
        .spacing(8.0)
        .build();

        let popover_content = Element::column([
            text("Start a new task with Copilot", 15.0, theme.foreground, 600),
            text(
                "Describe your task in natural language.",
                13.0,
                theme.muted_foreground,
                400,
            ),
            TextArea::new(
                "group-copilot-task",
                &self.button_group.task,
                "I need to...",
                theme.input(),
            )
            .build()
            .height(length(92.0)),
            text(
                "Copilot will open a pull request for review.",
                12.0,
                theme.muted_foreground,
                400,
            ),
            button("copilot-start", "Start task", theme.button()),
        ])
        .gap(10.0);
        let popover_trigger = symbol_button("copilot-menu", "Open Copilot", "v", outline(), theme);
        let copilot_popover = Popover::new(
            COPILOT_POPOVER,
            "Open Copilot",
            self.button_group.overlay == Some(ButtonGroupOverlay::Popover),
            popover_trigger,
            popover_content,
        )
        .size(300.0, 300.0)
        .build(theme);
        let popover = ButtonGroup::new(
            "copilot-actions",
            "Copilot options",
            [button("copilot", "Copilot", outline()), copilot_popover],
        )
        .build();

        let rtl = Direction::new(
            WritingDirection::Rtl,
            ButtonGroup::new(
                "rtl-actions",
                "إجراءات الرسالة",
                [
                    button("archive-rtl", "أرشفة", outline()),
                    button("report-rtl", "تقرير", outline()),
                    button("snooze-rtl", "تأجيل", outline()),
                    symbol_button("more-rtl", "خيارات أخرى", "•••", outline(), theme),
                ],
            )
            .build(),
        )
        .build();

        Element::column([
            section("Overview", overview, theme),
            section("Orientation", orientation, theme),
            section("Size", sizes, theme),
            section("Nested", nested, theme),
            section("Separator", separated, theme),
            section("Split", split, theme),
            section("Input", input, theme),
            section("Input Group", input_group, theme),
            section("Dropdown Menu", dropdown, theme),
            section("Select", select, theme),
            section("Popover", popover, theme),
            section("RTL", rtl, theme),
        ])
        .gap(28.0)
        .max_width(length(680.0))
    }
}
