#[cfg(target_os = "linux")]
#[test]
fn entrypoint_reports_the_missing_graphical_backend_without_using_the_user_session() {
    use std::{process::Command, thread, time::Duration};
    use web_time::Instant;

    for profiling in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_argui-state"));
        command
            .env_remove("XDG_SEAT")
            .env_remove("XDG_VTNR")
            .env_remove("XDG_SESSION_ID")
            .env_remove("XDG_SESSION_DESKTOP")
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("WAYLAND_SOCKET")
            .env_remove("XDG_RUNTIME_DIR")
            .env_remove("XDG_SESSION_TYPE")
            .env_remove("XDG_CURRENT_DESKTOP")
            .env_remove("DESKTOP_SESSION")
            .env_remove("DBUS_SESSION_BUS_ADDRESS")
            .env_remove("DBUS_STARTER_ADDRESS")
            .env_remove("DBUS_STARTER_BUS_TYPE")
            .env_remove("GDK_BACKEND")
            .env_remove("XAUTHORITY")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        if profiling {
            command.env("ARGUI_PROFILE", "1");
        } else {
            command.env_remove("ARGUI_PROFILE");
        }

        let mut child = command.spawn().expect("state app binary should start");
        let deadline = Instant::now() + Duration::from_secs(5);
        let output = loop {
            if child
                .try_wait()
                .expect("child status should be readable")
                .is_some()
            {
                break child
                    .wait_with_output()
                    .expect("state app output should be readable");
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("state app did not reject a session without a display");
            }
            thread::sleep(Duration::from_millis(25));
        };
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
        assert!(
            ["display", "wayland", "x11", "event loop"]
                .iter()
                .any(|expected| stderr.contains(expected)),
            "expected a missing-display error, received: {stderr}"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn profiled_application_renders_and_closes_normally_on_a_private_display() {
    use std::{path::PathBuf, process::Command};

    if std::env::var_os("ARGUI_NATIVE_TESTS").is_none() {
        return;
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(root.join("scripts/linux-hidden-display.sh"))
        .current_dir(&root)
        .env("ARGUI_TEST_BACKEND", "x11")
        .arg("timeout")
        .arg("75s")
        .arg("python3")
        .arg("crates/argui-state-app/tests/main.py")
        .arg(env!("CARGO_BIN_EXE_argui-state"))
        .output()
        .expect("private-display helper should start");

    assert!(
        output.status.success(),
        "profiled app did not render and close normally; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
