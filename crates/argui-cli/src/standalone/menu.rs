//! Keyboard driven init choices for an interactive terminal.

use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

/// One visible row in a selection menu.
struct Choice {
    label: &'static str,
    detail: &'static str,
    enabled: bool,
    selected: bool,
}

/// Restores the caller's terminal mode when menu input ends.
struct RawTerminal {
    saved: String,
}

impl RawTerminal {
    /// Enables single-key input and records the current terminal mode.
    ///
    /// # Errors
    /// Returns an error if the terminal cannot enter raw mode.
    fn open() -> Result<Self, String> {
        let output = Command::new("stty")
            .arg("-g")
            .stdin(Stdio::inherit())
            .output()
            .map_err(|error| format!("interactive init needs a terminal: {error}"))?;
        if !output.status.success() {
            return Err(
                "interactive init needs a terminal with stty support; use --yes and flags".into(),
            );
        }
        let saved = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let status = Command::new("stty")
            .args(["raw", "-echo", "min", "0", "time", "1"])
            .status()
            .map_err(|error| format!("cannot enable terminal input: {error}"))?;
        if !status.success() {
            return Err("cannot enable raw terminal input".into());
        }
        print!("\x1b[?25l");
        Ok(Self { saved })
    }
}

impl Drop for RawTerminal {
    /// Restores the saved terminal flags and cursor visibility.
    fn drop(&mut self) {
        let _ = Command::new("stty").arg(&self.saved).status();
        println!("\x1b[?25h\x1b[0m");
        let _ = std::io::stdout().flush();
    }
}

/// Collects framework, target, and capability choices from a TTY.
///
/// # Errors
/// Returns an error on Escape, malformed input, or unavailable terminal control.
pub(super) fn choose(
    framework: Option<String>,
    targets: Option<Vec<String>>,
    features: Vec<String>,
) -> Result<(String, Vec<String>, Vec<String>), String> {
    let _terminal = RawTerminal::open()?;
    let framework = if let Some(framework) = framework {
        framework
    } else {
        let mut choices = [
            Choice {
                label: "Rust",
                detail: "Pure Rust application",
                enabled: true,
                selected: false,
            },
            Choice {
                label: "Solid",
                detail: "One Solid TSX scene for every selected target",
                enabled: true,
                selected: true,
            },
            Choice {
                label: "React",
                detail: "One React TSX scene for every selected target",
                enabled: true,
                selected: false,
            },
        ];
        let index = select("Language and framework", &mut choices, false)?;
        ["rust", "solid", "react"][index].to_owned()
    };
    let targets = if let Some(targets) = targets {
        targets
    } else {
        let mut choices = [
            Choice {
                label: "Desktop native",
                detail: "Desktop QuickJS host and native renderer",
                enabled: true,
                selected: true,
            },
            Choice {
                label: "Web",
                detail: "WASM renderer and browser page; requires Solid or React",
                enabled: framework != "rust",
                selected: framework != "rust",
            },
            Choice {
                label: "Mobile (Android + iOS)",
                detail: "Unavailable: iOS app shell and packaging are not implemented",
                enabled: false,
                selected: false,
            },
        ];
        select("Targets", &mut choices, true)?;
        choices
            .iter()
            .enumerate()
            .filter(|(_, row)| row.selected)
            .map(|(index, _)| ["native", "web", "mobile"][index].to_owned())
            .collect()
    };
    let native = targets.iter().any(|target| target == "native");
    let mut choices = [
        Choice {
            label: "Required: UI engine and theme",
            detail: "argui-runtime, UI, layout, renderer, platform, theme and core crates",
            enabled: false,
            selected: true,
        },
        Choice {
            label: "Required: TSX bridge",
            detail: "argui-host and schema; present for Solid and React",
            enabled: false,
            selected: framework != "rust",
        },
        Choice {
            label: "Automation & metrics",
            detail: "Native TSX tests and screenshots; enables runtime/inspect only for test builds",
            enabled: native && framework != "rust",
            selected: features.iter().any(|feature| feature == "automation"),
        },
        Choice {
            label: "Async tasks",
            detail: "argui-runtime/tasks on selected Rust hosts",
            enabled: true,
            selected: features.iter().any(|feature| feature == "tasks"),
        },
        Choice {
            label: "Internationalization",
            detail: "Unavailable in the standalone SDK snapshot",
            enabled: false,
            selected: false,
        },
        Choice {
            label: "Media",
            detail: "Unavailable as a selectable host capability",
            enabled: false,
            selected: false,
        },
        Choice {
            label: "Effects",
            detail: "Unavailable as a selectable host capability",
            enabled: false,
            selected: false,
        },
        Choice {
            label: "WebView",
            detail: "Unavailable in the generated host",
            enabled: false,
            selected: false,
        },
        Choice {
            label: "Updater",
            detail: "Unavailable in the generated host",
            enabled: false,
            selected: false,
        },
        Choice {
            label: "Desktop integrations",
            detail: "Unavailable as a selectable host capability",
            enabled: false,
            selected: false,
        },
    ];
    select("Capabilities", &mut choices, true)?;
    let features = [("automation", 2), ("tasks", 3)]
        .into_iter()
        .filter(|(_, index)| choices[*index].selected)
        .map(|(name, _)| name.to_owned())
        .collect::<Vec<_>>();
    print!(
        "\x1b[2J\x1b[HFramework: {framework}\nTargets: {}\nCapabilities: {}\nPress Enter to generate, Escape to cancel.\n",
        targets.join(", "),
        if features.is_empty() {
            "none".into()
        } else {
            features.join(", ")
        }
    );
    std::io::stdout()
        .flush()
        .map_err(|error| error.to_string())?;
    loop {
        match key()? {
            Key::Enter => break,
            Key::Escape => return Err("initialization cancelled".into()),
            _ => {}
        }
    }
    Ok((framework, targets, features))
}

/// Moves through and updates `choices`, returning the selected cursor row.
///
/// # Errors
/// Returns an error on Escape or terminal I/O failure.
fn select(title: &str, choices: &mut [Choice], multiple: bool) -> Result<usize, String> {
    let mut cursor = choices
        .iter()
        .position(|choice| choice.selected)
        .unwrap_or(0);
    let mut offset = 0;
    loop {
        let height = terminal_rows().saturating_sub(7).max(3);
        if cursor < offset {
            offset = cursor;
        }
        if cursor >= offset + height {
            offset = cursor + 1 - height;
        }
        print!("\x1b[2J\x1b[H{title}\n↑/↓ move  Space select  Enter continue  Esc cancel\n\n");
        for (index, choice) in choices.iter().enumerate().skip(offset).take(height) {
            let marker = if multiple {
                if choice.selected { "[x]" } else { "[ ]" }
            } else if choice.selected {
                "(*)"
            } else {
                "( )"
            };
            let disabled = if choice.enabled { "" } else { "  unavailable" };
            println!(
                "{} {marker} {}{disabled}",
                if index == cursor { ">" } else { " " },
                choice.label
            );
        }
        println!(
            "\n{}/{}  {}",
            cursor + 1,
            choices.len(),
            choices[cursor].detail
        );
        std::io::stdout()
            .flush()
            .map_err(|error| error.to_string())?;
        match key()? {
            Key::Up => cursor = cursor.saturating_sub(1),
            Key::Down => cursor = (cursor + 1).min(choices.len() - 1),
            Key::Space if choices[cursor].enabled => {
                if multiple {
                    choices[cursor].selected = !choices[cursor].selected;
                } else {
                    for row in choices.iter_mut() {
                        row.selected = false;
                    }
                    choices[cursor].selected = true;
                }
            }
            Key::Enter if !multiple && choices[cursor].enabled => return Ok(cursor),
            Key::Enter
                if multiple
                    && (title == "Capabilities"
                        || choices
                            .iter()
                            .any(|choice| choice.selected && choice.enabled)) =>
            {
                return Ok(cursor);
            }
            Key::Escape => return Err("initialization cancelled".into()),
            _ => {}
        }
    }
}

/// One menu input token.
enum Key {
    Up,
    Down,
    Space,
    Enter,
    Escape,
    Other,
}

/// Reads one key, recognizing ANSI arrow sequences.
///
/// # Errors
/// Returns an error if stdin closes or fails.
fn key() -> Result<Key, String> {
    let mut input = std::io::stdin();
    let mut byte = [0];
    while input.read(&mut byte).map_err(|error| error.to_string())? == 0 {}
    Ok(match byte[0] {
        b' ' => Key::Space,
        b'\r' | b'\n' => Key::Enter,
        3 => Key::Escape,
        27 => {
            if input.read(&mut byte).map_err(|error| error.to_string())? == 0 || byte[0] != b'[' {
                Key::Escape
            } else {
                if input.read(&mut byte).map_err(|error| error.to_string())? == 0 {
                    return Ok(Key::Escape);
                }
                match byte[0] {
                    b'A' => Key::Up,
                    b'B' => Key::Down,
                    _ => Key::Escape,
                }
            }
        }
        _ => Key::Other,
    })
}

/// Reads terminal height, returning a conservative default if unavailable.
fn terminal_rows() -> usize {
    Command::new("stty")
        .arg("size")
        .stdin(Stdio::inherit())
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|body| body.split_whitespace().next()?.parse().ok())
        .unwrap_or(20)
}
