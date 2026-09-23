//! Client connection, generation acknowledgement, and reload behavior.
#![cfg(not(target_arch = "wasm32"))]

mod client_behavior {
    use std::{
        collections::HashMap,
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
        time::Duration,
    };

    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_ir::IrProject;
    use argui_dsl_protocol::{
        DiagnosticMessage, LiveMessage, LivePackageEnvelope, PackageHeader, RuntimeVersions,
        Severity, read_frame, write_frame,
    };
    use argui_dsl_runtime::{ClientEvent, LiveClient, LivePackage, LiveRuntime, RuntimeError};
    use argui_testing::TestApp;

    fn compile(source: &str) -> CompiledProject {
        Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("client behavior tests do not load external assets".into()),
        )
        .unwrap()
    }

    fn empty_project() -> IrProject {
        IrProject {
            modules: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            components: Vec::new(),
            themes: Vec::new(),
            styles: Vec::new(),
            effects: Vec::new(),
            assets: Vec::new(),
        }
    }

    fn hello() -> LiveMessage {
        let versions = RuntimeVersions::current();
        LiveMessage::Hello {
            protocol_version: versions.protocol,
            ir_format_version: versions.ir,
            engine_version: versions.engine,
        }
    }

    fn server<F>(handler: F) -> (std::net::SocketAddr, thread::JoinHandle<()>)
    where
        F: FnOnce(&mut TcpStream) + Send + 'static,
    {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let task = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handler(&mut stream);
        });
        (address, task)
    }

    #[test]
    fn client_validates_hello_receives_events_times_out_and_acknowledges() {
        let (ready_sender, ready_receiver) = mpsc::channel();
        let diagnostic = DiagnosticMessage {
            path: Some("main.argui".into()),
            start: Some(1),
            end: Some(2),
            severity: Severity::Warning,
            code: "warning".into(),
            message: "check".into(),
        };
        let (address, task) = server(move |stream| {
            write_frame(stream, &hello()).unwrap();
            ready_sender.send(()).unwrap();
            thread::sleep(Duration::from_millis(30));
            write_frame(
                stream,
                &LiveMessage::Diagnostics {
                    generation: 3,
                    diagnostics: vec![diagnostic],
                },
            )
            .unwrap();
            let status: LiveMessage = read_frame(stream).unwrap();
            assert_eq!(status, LiveMessage::Committed { generation: 3 });
        });
        let client = LiveClient::connect(address).unwrap();
        ready_receiver.recv().unwrap();
        assert_eq!(client.receive_timeout(Duration::from_millis(1)), None);
        assert!(matches!(
            client.receive_timeout(Duration::from_millis(200)),
            Some(ClientEvent::Diagnostics { generation: 3, .. })
        ));
        client
            .acknowledge(&LiveMessage::Committed { generation: 3 })
            .unwrap();
        task.join().unwrap();
    }

    #[test]
    fn client_reports_disconnect_and_rejects_non_hello_or_incompatible_versions() {
        let (address, task) = server(|stream| {
            write_frame(
                stream,
                &LiveMessage::Diagnostics {
                    generation: 1,
                    diagnostics: Vec::new(),
                },
            )
            .unwrap();
        });
        let error = LiveClient::connect(address).err().unwrap();
        assert!(
            matches!(error, RuntimeError::IncompatiblePackage(message) if message.contains("hello"))
        );
        task.join().unwrap();

        let (address, task) = server(|stream| {
            let mut message = hello();
            if let LiveMessage::Hello {
                protocol_version, ..
            } = &mut message
            {
                *protocol_version += 1;
            }
            write_frame(stream, &message).unwrap();
        });
        let error = LiveClient::connect(address).err().unwrap();
        assert!(
            matches!(error, RuntimeError::IncompatiblePackage(message) if message.contains("protocol"))
        );
        task.join().unwrap();

        let (address, task) = server(|_stream| {});
        let client = LiveClient::connect(address);
        assert!(client.is_err());
        task.join().unwrap();
    }

    #[test]
    fn runtime_connect_reports_timeout_non_package_and_missing_root() {
        let (address, task) = server(|stream| {
            write_frame(stream, &hello()).unwrap();
            thread::sleep(Duration::from_millis(80));
        });
        let error = LiveRuntime::connect(address, Duration::from_millis(1))
            .err()
            .unwrap();
        assert!(
            matches!(error, RuntimeError::IncompatiblePackage(message) if message.contains("initial package"))
        );
        task.join().unwrap();

        let (address, task) = server(|stream| {
            write_frame(stream, &hello()).unwrap();
            write_frame(
                stream,
                &LiveMessage::Diagnostics {
                    generation: 1,
                    diagnostics: vec![DiagnosticMessage {
                        path: None,
                        start: None,
                        end: None,
                        severity: Severity::Error,
                        code: "compile".into(),
                        message: "not ready".into(),
                    }],
                },
            )
            .unwrap();
        });
        let error = LiveRuntime::connect(address, Duration::from_millis(200))
            .err()
            .unwrap();
        assert!(
            matches!(error, RuntimeError::IncompatiblePackage(message) if message.contains("not ready"))
        );
        task.join().unwrap();

        let envelope = LivePackageEnvelope {
            header: PackageHeader::current(1, 1),
            roots: Vec::new(),
            ir: empty_project(),
            assets: Vec::new(),
        };
        let (address, task) = server(move |stream| {
            write_frame(stream, &hello()).unwrap();
            write_frame(stream, &LiveMessage::Package(Box::new(envelope))).unwrap();
        });
        let error = LiveRuntime::connect(address, Duration::from_millis(200))
            .err()
            .unwrap();
        assert!(
            matches!(error, RuntimeError::IncompatiblePackage(message) if message.contains("at least one component"))
        );
        task.join().unwrap();
    }

    #[test]
    fn runtime_connect_mounts_the_first_export_and_acknowledges_the_generation() {
        let compiled = compile(
            r#"import { Text } from "@argui/ui"
export component Main { Text { content: "connected" } }
"#,
        );
        let envelope = LivePackageEnvelope {
            header: PackageHeader::current(compiled.public_api_hash, 7),
            roots: compiled.roots.clone(),
            ir: compiled.ir,
            assets: Vec::new(),
        };
        let root = envelope.roots[0];
        let (address, task) = server(move |stream| {
            write_frame(stream, &hello()).unwrap();
            write_frame(stream, &LiveMessage::Package(Box::new(envelope))).unwrap();
            let status: LiveMessage = read_frame(stream).unwrap();
            assert_eq!(status, LiveMessage::Committed { generation: 7 });
        });

        let runtime = LiveRuntime::connect(address, Duration::from_secs(2)).unwrap();
        assert_eq!(
            runtime.root(),
            Some(argui_dsl_runtime::InstanceId::from_raw(1))
        );
        assert!(
            runtime
                .ir()
                .components
                .iter()
                .any(|component| component.id == root)
        );
        assert_eq!(runtime.generation(), 7);
        assert_eq!(runtime.last_client_event(), None);
        task.join().unwrap();
    }

    #[test]
    fn retained_render_listener_applies_diagnostics_and_a_later_commit() {
        let compiled = compile(
            r#"import { Text } from "@argui/ui"
export component Main { Text { content: "live" } }
"#,
        );
        let first = LivePackageEnvelope {
            header: PackageHeader::current(compiled.public_api_hash, 1),
            roots: compiled.roots.clone(),
            ir: compiled.ir.clone(),
            assets: Vec::new(),
        };
        let second = LivePackageEnvelope {
            header: PackageHeader::current(compiled.public_api_hash, 2),
            roots: compiled.roots,
            ir: compiled.ir,
            assets: Vec::new(),
        };
        let (diagnostics_ready_sender, diagnostics_ready_receiver) = mpsc::channel();
        let (continue_sender, continue_receiver) = mpsc::channel();
        let (stop_sender, stop_receiver) = mpsc::channel();
        let (address, task) = server(move |stream| {
            write_frame(stream, &hello()).unwrap();
            write_frame(stream, &LiveMessage::Package(Box::new(first))).unwrap();
            let status: LiveMessage = read_frame(stream).unwrap();
            assert_eq!(status, LiveMessage::Committed { generation: 1 });
            write_frame(
                stream,
                &LiveMessage::Diagnostics {
                    generation: 1,
                    diagnostics: Vec::new(),
                },
            )
            .unwrap();
            diagnostics_ready_sender.send(()).unwrap();
            continue_receiver.recv().unwrap();
            write_frame(stream, &LiveMessage::Package(Box::new(second))).unwrap();
            let status: LiveMessage = read_frame(stream).unwrap();
            assert_eq!(status, LiveMessage::Committed { generation: 2 });
            stop_receiver.recv().unwrap();
        });

        let runtime = LiveRuntime::connect(address, Duration::from_secs(2)).unwrap();
        diagnostics_ready_receiver.recv().unwrap();
        let mut app = TestApp::new(runtime);
        // Pump the paused task runtime while the transport decodes each package.
        for _ in 0..400 {
            if app.entity().read(|runtime| {
                matches!(
                    runtime.last_client_event(),
                    Some(ClientEvent::Diagnostics { .. })
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(5));
            app.settle().unwrap();
        }
        assert!(app.entity().read(|runtime| {
            matches!(
                runtime.last_client_event(),
                Some(ClientEvent::Diagnostics { .. })
            )
        }));
        continue_sender.send(()).unwrap();

        for _ in 0..400 {
            if app.entity().read(|runtime| {
                matches!(
                    runtime.last_client_event(),
                    Some(ClientEvent::Committed(outcome)) if outcome.generation == 2
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(5));
            app.settle().unwrap();
        }
        assert_eq!(
            app.entity().read(LiveRuntime::generation),
            2,
            "last client event: {:?}",
            app.entity()
                .read(|runtime| runtime.last_client_event().cloned())
        );
        assert!(app.entity().read(|runtime| {
            matches!(
                runtime.last_client_event(),
                Some(ClientEvent::Committed(outcome)) if outcome.generation == 2
            )
        }));
        stop_sender.send(()).unwrap();
        drop(app);
        task.join().unwrap();
    }

    #[test]
    fn package_transport_helpers_keep_an_empty_runtime_usable() {
        let package = LivePackage::prepare(1, 1, empty_project(), HashMap::new()).unwrap();
        let runtime = LiveRuntime::new(package).unwrap();
        assert_eq!(runtime.root(), None);
    }
}

mod client_edges {
    use std::{
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
        time::Duration,
    };

    use argui_dsl_protocol::{DiagnosticMessage, LiveMessage, Severity, write_frame};
    use argui_dsl_runtime::{ClientEvent, LiveClient, LiveRuntime, RuntimeError};

    fn hello() -> LiveMessage {
        let versions = argui_dsl_protocol::RuntimeVersions::current();
        LiveMessage::Hello {
            protocol_version: versions.protocol,
            ir_format_version: versions.ir,
            engine_version: versions.engine,
        }
    }

    fn server<F>(handler: F) -> (std::net::SocketAddr, thread::JoinHandle<()>)
    where
        F: FnOnce(&mut TcpStream) + Send + 'static,
    {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let task = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handler(&mut stream);
        });
        (address, task)
    }

    fn diagnostic(generation: u64) -> LiveMessage {
        LiveMessage::Diagnostics {
            generation,
            diagnostics: vec![DiagnosticMessage {
                path: None,
                start: None,
                end: None,
                severity: Severity::Warning,
                code: "edge".into(),
                message: "edge".into(),
            }],
        }
    }

    #[test]
    fn receiver_reports_transport_close_and_acknowledge_maps_write_errors() {
        let (address, task) = server(|stream| {
            write_frame(stream, &hello()).unwrap();
        });
        let client = LiveClient::connect(address).unwrap();
        assert!(matches!(
            client.receive_timeout(Duration::from_millis(200)),
            Some(ClientEvent::Disconnected(_))
        ));
        assert!(matches!(
            client.acknowledge(&LiveMessage::Committed { generation: 1 }),
            Err(RuntimeError::IncompatiblePackage(_))
        ));
        task.join().unwrap();
    }

    #[test]
    fn reader_stops_cleanly_when_the_client_receiver_is_dropped() {
        let (ready, wait) = mpsc::channel();
        let (go, proceed) = mpsc::channel();
        let (address, task) = server(move |stream| {
            write_frame(stream, &hello()).unwrap();
            ready.send(()).unwrap();
            proceed.recv().unwrap();
            let _ = write_frame(stream, &diagnostic(2));
        });
        let client = LiveClient::connect(address).unwrap();
        wait.recv_timeout(Duration::from_secs(1)).unwrap();
        drop(client);
        go.send(()).unwrap();
        task.join().unwrap();
    }

    #[test]
    fn reader_preserves_order_before_reporting_a_later_disconnect() {
        let (address, task) = server(|stream| {
            write_frame(stream, &hello()).unwrap();
            write_frame(stream, &diagnostic(3)).unwrap();
        });
        let client = LiveClient::connect(address).unwrap();
        assert!(matches!(
            client.receive_timeout(Duration::from_millis(200)),
            Some(ClientEvent::Diagnostics { generation: 3, .. })
        ));
        assert!(matches!(
            client.receive_timeout(Duration::from_millis(200)),
            Some(ClientEvent::Disconnected(_))
        ));
        task.join().unwrap();
    }

    #[test]
    fn client_reports_a_broken_hello_frame_without_panicking() {
        let (address, task) = server(|_stream| {});
        let error = match LiveClient::connect(address) {
            Ok(_) => panic!("a broken hello frame must be rejected"),
            Err(error) => error,
        };
        assert!(matches!(error, RuntimeError::IncompatiblePackage(_)));
        task.join().unwrap();
    }

    #[test]
    fn reader_translates_wire_statuses_without_losing_their_order() {
        let (address, task) = server(|stream| {
            write_frame(stream, &hello()).unwrap();
            for message in [
                LiveMessage::RestartRequired {
                    generation: 4,
                    previous_api_hash: 10,
                    next_api_hash: 11,
                },
                LiveMessage::Rejected {
                    generation: 5,
                    message: "invalid asset".into(),
                },
                LiveMessage::Committed { generation: 6 },
                hello(),
            ] {
                write_frame(stream, &message).unwrap();
            }
        });
        let client = LiveClient::connect(address).unwrap();
        assert_eq!(
            client.receive_timeout(Duration::from_secs(1)),
            Some(ClientEvent::RestartRequired {
                generation: 4,
                previous_api_hash: 10,
                next_api_hash: 11,
            })
        );
        let Some(ClientEvent::Diagnostics {
            generation,
            diagnostics,
        }) = client.receive_timeout(Duration::from_secs(1))
        else {
            panic!("rejection must become a diagnostic event");
        };
        assert_eq!(generation, 5);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "rejected");
        assert_eq!(diagnostics[0].message, "invalid asset");
        assert_eq!(diagnostics[0].severity, Severity::Error);
        assert!(matches!(
            client.receive_timeout(Duration::from_secs(1)),
            Some(ClientEvent::Diagnostics {
                generation: 6,
                diagnostics,
            }) if diagnostics.is_empty()
        ));
        assert_eq!(
            client.receive_timeout(Duration::from_secs(1)),
            Some(ClientEvent::Disconnected(
                "unexpected repeated hello frame".into()
            ))
        );
        task.join().unwrap();
    }

    #[test]
    fn initial_restart_status_reports_the_host_request() {
        let (address, task) = server(|stream| {
            write_frame(stream, &hello()).unwrap();
            write_frame(
                stream,
                &LiveMessage::RestartRequired {
                    generation: 2,
                    previous_api_hash: 3,
                    next_api_hash: 4,
                },
            )
            .unwrap();
        });
        let result = LiveRuntime::connect(address, Duration::from_secs(1));
        assert!(matches!(
            result,
            Err(RuntimeError::IncompatiblePackage(message))
                if message.contains("requested a restart")
        ));
        task.join().unwrap();
    }
}

mod reload_behavior {
    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_protocol::{LivePackageEnvelope, PackageHeader};
    use argui_dsl_runtime::{ClientEvent, LiveClient, LivePackage, LiveRuntime};
    use argui_testing::TestApp;

    /// Compiles one source snapshot without requiring project assets.
    fn compile(source: &str) -> CompiledProject {
        Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("reload tests do not load external assets".into()),
        )
        .unwrap()
    }

    /// Creates the wire package used by one live compiler generation.
    fn envelope(generation: u64, title: &str) -> (LivePackageEnvelope, argui_dsl_ir::ComponentId) {
        let compiled = compile(&format!(
            r#"import {{ Text }} from "@argui/ui"
export component Main {{
    in property title: string = "{title}"
    Text {{ content: title }}
}}
"#
        ));
        let root = compiled.roots[0];
        (
            LivePackageEnvelope {
                header: PackageHeader::current(compiled.public_api_hash, generation),
                roots: compiled.roots,
                ir: compiled.ir,
                assets: Vec::new(),
            },
            root,
        )
    }

    /// Applies a package through the public client path and renders its changed default.
    #[test]
    fn client_apply_reload_updates_default_title_in_rendered_tree() {
        let (first_envelope, root) = envelope(1, "before");
        let (second_envelope, _) = envelope(2, "after");
        let first = LivePackage::from_envelope(first_envelope).unwrap();
        let mut runtime = LiveRuntime::new(first).unwrap();
        runtime.mount(root, []).unwrap();

        let committed = LiveClient::apply(
            &mut runtime,
            ClientEvent::Package(Box::new(second_envelope)),
        )
        .unwrap();
        assert!(matches!(committed, ClientEvent::Committed(_)));
        assert_eq!(runtime.generation(), 2);

        TestApp::new(runtime).assert_text("after");
    }
}
