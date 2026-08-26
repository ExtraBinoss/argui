use argui_ui::{Element, UiEvent};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewUpdate {
    #[default]
    None,
    Rebuild,
}

pub trait UiApp: 'static {
    fn view(&self) -> Element;

    fn update(&mut self, event: &UiEvent) -> ViewUpdate;
}
