use argui_animation::Frame;
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

    fn animation_frame(&mut self, _frame: Frame) -> ViewUpdate {
        ViewUpdate::None
    }

    fn wants_animation_frame(&self) -> bool {
        false
    }
}
