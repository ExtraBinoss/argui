#![cfg(not(target_arch = "wasm32"))]

use std::{net::SocketAddr, path::Path};

use argui_cli::{
    SourceChange,
    dashboard::{DevConsole, DevDashboard, DevPhase},
};
use argui_dsl_protocol::{
    DiagnosticMessage, LiveMessage, LivePackageEnvelope, PackageHeader, Severity,
};
use ratatui::{Terminal, backend::TestBackend, style::Color};

/// Renders a dashboard into a deterministic virtual terminal for assertions.
fn rendered(dashboard: &DevDashboard, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| dashboard.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

/// A real source delta, client acknowledgement, and connection remain visible.
#[test]
fn dashboard_shows_live_change_and_confirmed_generation() {
    let native: SocketAddr = "127.0.0.1:4777".parse().unwrap();
    let browser: SocketAddr = "127.0.0.1:4778".parse().unwrap();
    let peer: SocketAddr = "127.0.0.1:50666".parse().unwrap();
    let mut dashboard = DevDashboard::new(Path::new("project"), native, browser);
    assert_eq!(dashboard.phase(), DevPhase::Watching);
    dashboard.change(&SourceChange {
        path: "ui/main.argui".into(),
        line: Some(4),
        column: Some(44),
        before: "title: string = \"before\"".into(),
        after: "title: string = \"after\"".into(),
    });
    assert_eq!(dashboard.phase(), DevPhase::Changed);
    dashboard.clients(1);
    dashboard.connected(peer);
    dashboard.client_status(peer, &LiveMessage::Committed { generation: 8 });
    assert_eq!(dashboard.phase(), DevPhase::Applied);
    let screen = rendered(&dashboard, 110, 20);
    assert!(screen.contains("ARGUI"));
    assert!(screen.contains("APPLIED"));
    assert!(screen.contains("ui/main.argui:4:44"));
    assert!(screen.contains("Generation 8 applied"));
    assert!(screen.contains("1 native"));
}

/// Rejected updates use the terminal error state, including client rejections.
#[test]
fn dashboard_shows_client_rejection_and_survives_small_terminals() {
    let native: SocketAddr = "127.0.0.1:4777".parse().unwrap();
    let browser: SocketAddr = "127.0.0.1:4778".parse().unwrap();
    let mut dashboard = DevDashboard::new(Path::new("project"), native, browser);
    dashboard.client_status(
        native,
        &LiveMessage::Rejected {
            generation: 9,
            message: "incompatible property".into(),
        },
    );
    assert_eq!(dashboard.phase(), DevPhase::Rejected);
    assert_eq!(dashboard.phase().ratio(), 1.0);
    assert!(rendered(&dashboard, 80, 18).contains("incompatible property"));
    let _ = rendered(&dashboard, 12, 4);
}

/// Builds a minimal package so phase transitions can be tested without a compiler.
fn package(generation: u64) -> LiveMessage {
    LiveMessage::Package(Box::new(LivePackageEnvelope {
        header: PackageHeader::current(42, generation),
        roots: Vec::new(),
        ir: argui_dsl_ir::IrProject {
            modules: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            components: Vec::new(),
            themes: Vec::new(),
            styles: Vec::new(),
            effects: Vec::new(),
            assets: Vec::new(),
        },
        assets: Vec::new(),
    }))
}

#[test]
fn dashboard_tracks_compilation_diagnostics_and_all_client_statuses() {
    let native: SocketAddr = "127.0.0.1:4777".parse().unwrap();
    let browser: SocketAddr = "127.0.0.1:4778".parse().unwrap();
    let peer: SocketAddr = "127.0.0.1:50666".parse().unwrap();
    let mut dashboard = DevDashboard::new(Path::new("project"), native, browser);

    assert_eq!(DevPhase::Watching.ratio(), 0.0);
    assert_eq!(DevPhase::Changed.ratio(), 0.25);
    assert_eq!(DevPhase::Compiled.ratio(), 0.65);
    assert_eq!(DevPhase::Applied.ratio(), 1.0);
    assert_eq!(DevPhase::Rejected.ratio(), 1.0);
    assert_eq!(DevPhase::Watching.presentation().0, "WATCHING");
    assert_eq!(DevPhase::Changed.presentation().0, "CHANGE DETECTED");
    assert_eq!(
        DevPhase::Compiled.presentation().0,
        "COMPILED · AWAITING CLIENT"
    );
    assert_eq!(DevPhase::Applied.presentation().0, "APPLIED");
    assert_eq!(DevPhase::Rejected.presentation().0, "REJECTED");

    let compiled = package(3);
    dashboard.compilation(&compiled);
    assert_eq!(dashboard.phase(), DevPhase::Compiled);
    assert!(rendered(&dashboard, 100, 20).contains("Generation 3 compiled"));

    dashboard.client_status(peer, &LiveMessage::Committed { generation: 2 });
    assert_eq!(dashboard.phase(), DevPhase::Compiled);
    assert!(rendered(&dashboard, 100, 20).contains("Delayed generation 2 status"));

    dashboard.compilation(&LiveMessage::Diagnostics {
        generation: 4,
        diagnostics: vec![DiagnosticMessage {
            path: Some("ui/main.argui".into()),
            start: Some(2),
            end: Some(5),
            severity: Severity::Warning,
            code: "MissingRepeaterKey".into(),
            message: "add a key".into(),
        }],
    });
    assert_eq!(dashboard.phase(), DevPhase::Rejected);
    assert!(rendered(&dashboard, 100, 20).contains("MissingRepeaterKey: add a key"));

    dashboard.compilation(&LiveMessage::Hello {
        protocol_version: 2,
        ir_format_version: 1,
        engine_version: "test".into(),
    });
    assert_eq!(dashboard.phase(), DevPhase::Rejected);

    dashboard.client_status(peer, &LiveMessage::Committed { generation: 5 });
    assert_eq!(dashboard.phase(), DevPhase::Applied);
    dashboard.client_status(
        peer,
        &LiveMessage::Rejected {
            generation: 6,
            message: "ABI changed".into(),
        },
    );
    assert_eq!(dashboard.phase(), DevPhase::Rejected);
    dashboard.client_status(
        peer,
        &LiveMessage::RestartRequired {
            generation: 7,
            previous_api_hash: 1,
            next_api_hash: 2,
        },
    );
    assert_eq!(dashboard.phase(), DevPhase::Rejected);
    dashboard.client_status(
        peer,
        &LiveMessage::Hello {
            protocol_version: 2,
            ir_format_version: 1,
            engine_version: "test".into(),
        },
    );
    assert!(rendered(&dashboard, 110, 20).contains("Unexpected client status"));
}

#[test]
fn dashboard_history_is_bounded_and_unlocated_changes_are_rendered() {
    let native: SocketAddr = "127.0.0.1:4777".parse().unwrap();
    let browser: SocketAddr = "127.0.0.1:4778".parse().unwrap();
    let mut dashboard = DevDashboard::new(Path::new("project"), native, browser);
    dashboard.change(&SourceChange {
        path: "assets/logo.png".into(),
        line: None,
        column: None,
        before: "old bytes".into(),
        after: "new bytes".into(),
    });
    assert!(rendered(&dashboard, 100, 20).contains("assets/logo.png"));
    for index in 0..70 {
        dashboard.note(format!("event-{index}"), Color::White);
    }
    let screen = rendered(&dashboard, 100, 90);
    assert!(screen.contains("event-69"));
    assert!(!screen.contains("event-0"));
}

/// Preserves paths when only one source location coordinate is available.
#[test]
fn dashboard_renders_partially_located_changes_and_empty_compilations() {
    let native: SocketAddr = "127.0.0.1:4777".parse().unwrap();
    let browser: SocketAddr = "127.0.0.1:4778".parse().unwrap();
    let mut dashboard = DevDashboard::new(Path::new("project"), native, browser);
    dashboard.change(&SourceChange {
        path: "ui/line-only.argui".into(),
        line: Some(7),
        column: None,
        before: "before".into(),
        after: "after".into(),
    });
    dashboard.change(&SourceChange {
        path: "ui/column-only.argui".into(),
        line: None,
        column: Some(3),
        before: "before".into(),
        after: "after".into(),
    });
    dashboard.compilation(&LiveMessage::Diagnostics {
        generation: 1,
        diagnostics: Vec::new(),
    });
    assert_eq!(dashboard.phase(), DevPhase::Rejected);
    let screen = rendered(&dashboard, 110, 20);
    assert!(screen.contains("ui/line-only.argui"));
    assert!(screen.contains("ui/column-only.argui"));
}

#[test]
fn noninteractive_console_forwards_events_to_the_dashboard() {
    let native: SocketAddr = "127.0.0.1:4777".parse().unwrap();
    let browser: SocketAddr = "127.0.0.1:4778".parse().unwrap();
    let peer: SocketAddr = "127.0.0.1:50666".parse().unwrap();
    let mut console = DevConsole::new(Path::new("project"), native, browser).unwrap();
    console
        .change(&SourceChange {
            path: "main.argui".into(),
            line: Some(1),
            column: Some(1),
            before: "before".into(),
            after: "after".into(),
        })
        .unwrap();
    console
        .change(&SourceChange {
            path: "assets/logo.png".into(),
            line: None,
            column: None,
            before: "old bytes".into(),
            after: "new bytes".into(),
        })
        .unwrap();
    console.compilation(&package(2)).unwrap();
    console
        .compilation(&LiveMessage::Diagnostics {
            generation: 3,
            diagnostics: vec![DiagnosticMessage {
                path: None,
                start: None,
                end: None,
                severity: Severity::Error,
                code: "parse".into(),
                message: "invalid syntax".into(),
            }],
        })
        .unwrap();
    console
        .compilation(&LiveMessage::Hello {
            protocol_version: 2,
            ir_format_version: 1,
            engine_version: "test".into(),
        })
        .unwrap();
    console
        .client_status(peer, &LiveMessage::Committed { generation: 4 })
        .unwrap();
    console
        .client_status(
            peer,
            &LiveMessage::Rejected {
                generation: 5,
                message: "rejected".into(),
            },
        )
        .unwrap();
    console
        .client_status(
            peer,
            &LiveMessage::RestartRequired {
                generation: 6,
                previous_api_hash: 1,
                next_api_hash: 2,
            },
        )
        .unwrap();
    console
        .client_status(
            peer,
            &LiveMessage::Hello {
                protocol_version: 2,
                ir_format_version: 1,
                engine_version: "test".into(),
            },
        )
        .unwrap();
    console.clients(1).unwrap();
    console.clients(1).unwrap();
    console.connected(peer).unwrap();
    console
        .notes(
            ["batch one", "batch two"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>(),
            Color::Blue,
        )
        .unwrap();
    console.notes(Vec::<String>::new(), Color::Blue).unwrap();
    console.note("custom note", Color::Magenta).unwrap();
}

/// Keeps no-op console updates side-effect free while accepting later changes.
#[test]
fn noninteractive_console_skips_empty_batches_and_duplicate_client_counts() {
    let native: SocketAddr = "127.0.0.1:4777".parse().unwrap();
    let browser: SocketAddr = "127.0.0.1:4778".parse().unwrap();
    let mut console = DevConsole::new(Path::new("project"), native, browser).unwrap();

    console.notes(Vec::<String>::new(), Color::Blue).unwrap();
    console.clients(0).unwrap();
    console.clients(0).unwrap();
    console.clients(2).unwrap();
    console.clients(2).unwrap();
}
