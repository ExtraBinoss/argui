use std::{env, fmt::Write, fs, path::PathBuf};

/// Generates the pinned Tabler lookup from the checked-in upstream symbol list.
///
/// # Panics
///
/// Panics if the manifest is malformed or Cargo's output directory is unavailable.
fn main() {
    println!("cargo:rerun-if-changed=icons/catalog.txt");
    let names = include_str!("icons/catalog.txt");
    let mut output = String::from(
        "/// Resolves a pinned Tabler name to its upstream glyph data.\n///\n/// * `name` — exported PascalCase icon name.\n///\n/// Returns `None` if the catalog has no such icon.\nfn icon_data(name: &str) -> Option<&'static icondata_core::IconData> {\n    match name {\n",
    );
    let mut previous = "";
    for name in names.lines() {
        assert!(
            !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric()),
            "invalid Tabler icon name: {name}"
        );
        assert!(
            previous < name,
            "Tabler icon names must be sorted and unique"
        );
        previous = name;
        let symbol = if name.ends_with("Filled") {
            format!("Tb{name}")
        } else {
            format!("Tb{name}Outline")
        };
        writeln!(output, "        \"{name}\" => Some(icondata_tb::{symbol}),").unwrap();
    }
    output.push_str("        _ => None,\n    }\n}\n");
    let path = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR"));
    fs::write(path.join("icons.rs"), output).expect("generated Tabler lookup is writable");
}
