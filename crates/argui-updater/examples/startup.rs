//! `cargo run -p argui-updater --features native --example startup -- URL update.pub`
//! Checks on a worker, delivering states through a channel. Does not install anything.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use argui_updater::{
        Updater,
        http::{Config, HttpBackend},
        install::NativeInstaller,
    };
    let mut args = std::env::args().skip(1);
    let endpoint = args.next().ok_or("expected manifest URL")?;
    let key_file = args.next().ok_or("expected Minisign public key file")?;
    let public_key = std::fs::read_to_string(key_file)?;
    let config = Config::new(env!("CARGO_PKG_VERSION"), &endpoint, &public_key)?;
    let (events, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut updater = Updater::new(HttpBackend::new(config, NativeInstaller::detect()?)?);
        updater.check(|state| {
            let _ = events.send(state.clone());
        })
    });
    for state in receiver {
        println!("{state:?}");
    }
    worker.join().map_err(|_| "update worker panicked")??;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("the native startup example requires a desktop target".into())
}
