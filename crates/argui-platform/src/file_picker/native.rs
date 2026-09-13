use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
impl FileDialogBackend for NativeFileDialog {
    fn open(&self, dialog: FileDialog) -> FileDialogFuture {
        if let Err(error) = dialog.validate() {
            return Box::pin(async { Err(error) });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (sender, receiver) = futures_channel::oneshot::channel();
            let worker = std::thread::Builder::new()
                .name("argui-file-dialog".into())
                .spawn(move || {
                    let result = pollster::block_on(selection(&dialog));
                    drop(dialog.parent);
                    let _ = sender.send(result);
                });
            Box::pin(async move {
                worker.map_err(|error| FileDialogError::Unavailable(error.to_string()))?;
                receiver.await.map_err(|_| {
                    FileDialogError::Unavailable(
                        "The native file dialog stopped unexpectedly".into(),
                    )
                })?
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            selection(&dialog)
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn selection(dialog: &FileDialog) -> FileDialogFuture {
    #[cfg(any(
        target_arch = "wasm32",
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))]
    {
        let mut native = rfd::AsyncFileDialog::new().set_title(&dialog.title);
        for filter in &dialog.filters {
            native = native.add_filter(&filter.name, &filter.extensions);
        }
        if let Some(directory) = &dialog.directory {
            native = native.set_directory(directory);
        }
        if let Some(name) = &dialog.file_name {
            native = native.set_file_name(name);
        }
        if let Some(parent) = &dialog.parent {
            if parent.window_handle().is_err() || parent.display_handle().is_err() {
                return Box::pin(async {
                    Err(FileDialogError::Unavailable(
                        "The dialog parent is unavailable".into(),
                    ))
                });
            }
            native = native.set_parent(parent.as_ref());
        }
        // RFD dispatches AppKit work onto its application event loop.
        match dialog.mode {
            FilePickerMode::File => {
                let future = native.pick_file();
                Box::pin(async { Ok(future.await.map(|file| vec![file])) })
            }
            FilePickerMode::Files => {
                let future = native.pick_files();
                Box::pin(async { Ok(future.await) })
            }
            #[cfg(not(target_arch = "wasm32"))]
            FilePickerMode::Folder => {
                let future = native.pick_folder();
                Box::pin(async { Ok(future.await.map(|file| vec![file])) })
            }
            #[cfg(not(target_arch = "wasm32"))]
            FilePickerMode::Folders => {
                let future = native.pick_folders();
                Box::pin(async { Ok(future.await) })
            }
            #[cfg(not(target_arch = "wasm32"))]
            FilePickerMode::Save => {
                let future = native.save_file();
                Box::pin(async { Ok(future.await.map(|file| vec![file])) })
            }
            #[cfg(target_arch = "wasm32")]
            mode => Box::pin(async move { Err(FileDialogError::UnsupportedMode(mode)) }),
        }
    }
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(any(target_os = "linux", target_os = "windows", target_os = "macos"))
    ))]
    {
        let mode = dialog.mode;
        Box::pin(async move { Err(FileDialogError::UnsupportedMode(mode)) })
    }
}
