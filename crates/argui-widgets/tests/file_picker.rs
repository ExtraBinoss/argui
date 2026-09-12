#![cfg(all(feature = "file-picker", not(target_arch = "wasm32")))]
use argui_platform::file_picker::*;
use argui_runtime::{Entity, tasks::TaskRuntime};
use argui_ui::{ClickEvent, Element, UiEventKind, UiTree};
use argui_widgets::{FilePicker, FilePickerStatus};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

struct Provider(Mutex<VecDeque<FileDialogResult>>);
impl FileDialogBackend for Provider {
    fn open(&self, dialog: FileDialog) -> FileDialogFuture {
        assert_eq!(dialog.title, "Test picker");
        let result = self
            .0
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected dialog");
        Box::pin(async { result })
    }
}
fn click(picker: &Entity<FilePicker>, key: &str) {
    let mut ui = UiTree::new(picker.render());
    let node = ui
        .node_ids()
        .iter()
        .copied()
        .find(|node| ui.key(*node) == Some(key))
        .unwrap();
    for event in ui.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            picker.dispatch_event(&event);
        }
    }
}
fn find<'a>(element: &'a Element, key: &str) -> &'a Element {
    fn search<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
        if element.key.as_deref() == Some(key) {
            return Some(element);
        }
        element.children.iter().find_map(|child| search(child, key))
    }
    search(element, key).unwrap()
}
fn drain(tasks: &TaskRuntime, wake: &mpsc::Receiver<()>) {
    while tasks.pending() > 0 {
        wake.recv_timeout(Duration::from_secs(5)).unwrap();
        tasks.drain();
    }
}

#[test]
fn selecting_dismissing_and_failing_preserve_real_committed_files() {
    let file: PickedFile = std::env::temp_dir().join("résumé.txt").into();
    let provider = Arc::new(Provider(Mutex::new(VecDeque::from([
        Ok(Some(vec![file.clone()])),
        Ok(None),
        Ok(Some(Vec::new())),
        Err(FileDialogError::Unavailable("Portal unavailable".into())),
    ]))));
    let picker = Entity::new(
        FilePicker::new(
            "pick",
            "Open document",
            FileDialog::default().title("Test picker"),
        )
        .backend(provider.clone()),
    );
    let (sender, wake) = mpsc::channel();
    let tasks = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    picker.set_task_runtime(tasks.clone());
    assert_eq!(
        picker.read(|picker| picker.status().clone()),
        FilePickerStatus::Idle
    );
    assert!(picker.read(|picker| picker.selection().is_empty()));
    for expected in [
        FilePickerStatus::Selected,
        FilePickerStatus::Dismissed,
        FilePickerStatus::Dismissed,
        FilePickerStatus::Failed("Portal unavailable".into()),
    ] {
        click(&picker, "pick");
        assert_eq!(
            picker.read(|picker| picker.status().clone()),
            FilePickerStatus::Opening
        );
        let opening = picker.render();
        assert!(
            find(&opening, "pick")
                .semantics
                .as_ref()
                .unwrap()
                .state
                .busy
        );
        click(&picker, "pick");
        drain(&tasks, &wake);
        assert_eq!(picker.read(|picker| picker.status().clone()), expected);
        assert_eq!(
            picker.read(|picker| picker.selection()[0].path().to_path_buf()),
            file.path()
        );
        assert!(format!("{:?}", picker.render()).contains("résumé.txt"));
    }
    assert!(provider.0.lock().unwrap().is_empty());
}

#[test]
fn disabled_invalid_and_executor_unavailable_do_not_open_a_dialog() {
    let provider = Arc::new(Provider(Mutex::new(VecDeque::new())));
    let picker =
        Entity::new(FilePicker::new("pick", "Open", FileDialog::default()).backend(provider));
    picker.update(|picker, cx| {
        picker.enabled = false;
        cx.notify();
    });
    click(&picker, "pick");
    assert_eq!(
        picker.read(|picker| picker.status().clone()),
        FilePickerStatus::Idle
    );
    assert!(
        find(&picker.render(), "pick")
            .semantics
            .as_ref()
            .unwrap()
            .state
            .disabled
    );
    picker.update(|picker, cx| {
        picker.enabled = true;
        picker.dialog.title = "bad\0title".into();
        cx.notify();
    });
    click(&picker, "pick");
    assert!(matches!(
        picker.read(|picker| picker.status().clone()),
        FilePickerStatus::Failed(_)
    ));
    picker.update(|picker, cx| {
        picker.dialog.title = "valid".into();
        cx.notify();
    });
    click(&picker, "pick");
    assert_eq!(
        picker.read(|picker| picker.status().clone()),
        FilePickerStatus::Failed("Unavailable".into())
    );
    assert_eq!(
        find(&picker.render(), "pick::status")
            .semantics
            .as_ref()
            .unwrap()
            .role,
        argui_ui::Role::Alert
    );
}

#[test]
fn subscribers_receive_selection_and_dismissal_events() {
    use argui_widgets::FilePickerEvent;
    let models = argui_runtime::ModelRuntime::default();
    let observer = models.entity(Vec::<String>::new());
    let provider = Arc::new(Provider(Mutex::new(VecDeque::from([
        Ok(Some(vec![std::env::temp_dir().join("report.txt").into()])),
        Ok(None),
        Err(FileDialogError::Unavailable("No service".into())),
    ]))));
    let picker = models.entity(
        FilePicker::new("pick", "Open", FileDialog::default().title("Test picker"))
            .backend(provider),
    );
    let _subscription = observer
        .update(|_, cx| {
            cx.subscribe(&picker, |events, event, _| {
                events.push(match event {
                    FilePickerEvent::Selected(files) => files[0].file_name(),
                    FilePickerEvent::Dismissed => "dismissed".into(),
                    FilePickerEvent::Failed(error) => error.to_string(),
                });
            })
        })
        .unwrap();
    let (sender, wake) = mpsc::channel();
    let tasks = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    models.set_task_runtime(tasks.clone());
    for _ in 0..3 {
        click(&picker, "pick");
        drain(&tasks, &wake);
        models.dispatch_pending();
    }
    assert_eq!(
        observer.read(Clone::clone),
        ["report.txt", "dismissed", "No service"]
    );
}
