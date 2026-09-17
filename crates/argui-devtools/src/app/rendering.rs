use super::{AppModel, DevtoolsApp, Element, WindowEnvironment, WindowKey, view};

impl<M: AppModel> DevtoolsApp<M> {
    pub(super) fn view_window(
        &self,
        window: &WindowKey,
        environment: WindowEnvironment,
    ) -> Option<Element> {
        let safe_area = environment.safe_area_insets;
        {
            let mut tools = self.tools.borrow_mut();
            tools.reduced_motion = environment.reduced_motion;
            if window == &self.target {
                tools.theme_editing.capture(&environment);
            }
            if tools.reduced_motion {
                let open = tools.dock_presence.is_open();
                tools.dock_presence.set_open(open, true);
            }
        }
        let themes = argui_widgets::shadcn(&environment);
        let theme = themes.resolve(environment.color_scheme);
        if window == &self.detached {
            let tools = self.tools.borrow();
            return Some(
                view::dock(&tools, tools.viewport.size.height.max(1.0), theme)
                    .width(argui_ui::percent(1.0)),
            );
        }
        let environment = if window == &self.target {
            self.tools.borrow().theme_editing.environment()
        } else {
            environment
        };
        let app = self.app.view(window, environment)?;
        if window != &self.target {
            return Some(app);
        }
        let tools = self.tools.borrow();
        let mut root = view::host(&tools, app, theme, None, safe_area);
        if let Some(error) = &self.error {
            root.children.push(Element::text(format!(
                "Could not open developer tools window: {error}"
            )));
        }
        Some(root)
    }
}
