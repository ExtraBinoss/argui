export const repository = 'https://github.com/ExtraBinoss/argui'
export const discord = 'https://discord.gg/xY9CWSc65'
export const currentVersion = '0.3.2'
export const sourceUrl = (path: string) => `${repository}/blob/main/${path}`
export const composeExample = `use argui::{
    ui::Element,
    widgets::{Button, WidgetTheme},
};

fn view(theme: &WidgetTheme) -> Element {
    Element::row([
        Button::new("save", "Save changes", theme.button())
            .build(),
        Button::new("cancel", "Cancel", theme.ghost_button())
            .build(),
    ])
    .gap(10.0)
}`
