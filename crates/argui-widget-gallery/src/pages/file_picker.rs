use crate::app::text;
use argui::{
    platform::file_picker::{FileDialog, FileFilter, FilePickerMode},
    runtime::{Context, Entity, Render},
    ui::{Element, length, percent},
    widgets::{FilePicker, shadcn},
};

pub(crate) struct FilePickerDemo {
    pickers: Vec<(&'static str, &'static str, Entity<FilePicker>)>,
}
impl Default for FilePickerDemo {
    fn default() -> Self {
        let examples = [
            (
                "file-open",
                "Open a document",
                "Choose one document. Its path appears below.",
                FileDialog::new(FilePickerMode::File)
                    .title("Open a document")
                    .filter(FileFilter::new("Documents", ["txt", "md", "pdf"])),
            ),
            (
                "files-open",
                "Select images",
                "Choose several images in a single dialog.",
                FileDialog::new(FilePickerMode::Files)
                    .title("Select images")
                    .filter(FileFilter::new(
                        "Images",
                        ["png", "jpg", "jpeg", "webp", "svg"],
                    )),
            ),
            (
                "folder-open",
                "Choose project folder",
                "Select the folder that contains your project.",
                FileDialog::new(FilePickerMode::Folder).title("Choose a project folder"),
            ),
            (
                "folders-open",
                "Choose source folders",
                "Select several folders to include in a collection.",
                FileDialog::new(FilePickerMode::Folders).title("Choose source folders"),
            ),
            (
                "file-save",
                "Choose export destination",
                "Choose where to save a report. This example selects a destination without writing a file.",
                FileDialog::new(FilePickerMode::Save)
                    .title("Export report")
                    .file_name("report.txt")
                    .filter(FileFilter::new("Text", ["txt"])),
            ),
        ];
        Self {
            pickers: examples
                .into_iter()
                .map(|(key, title, description, dialog)| {
                    (
                        title,
                        description,
                        Entity::new(FilePicker::new(key, title, dialog)),
                    )
                })
                .collect(),
        }
    }
}
impl Render for FilePickerDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        Element::column(self.pickers.iter().map(|(title, description, picker)| {
            Element::column([
                text(*title, 16.0, theme.foreground, 600),
                text(*description, 13.0, theme.muted_foreground, 400),
                cx.entity(picker),
            ])
            .gap(10.0)
            .padding(argui::ui::Sides::length(18.0))
            .background(theme.card)
            .border(argui::paint::Border::all(1.0, theme.border))
            .radius(argui::paint::CornerRadii::all(10.0))
        }))
        .gap(18.0)
        .width(percent(1.0))
        .max_width(length(720.0))
    }
}
