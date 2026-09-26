//! Desktop automation command routing and process monitoring.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use argui_automation::{ProcessSample, ProcessSampler};
use serde_json::{Value, json};

use crate::standalone;

/// Builds and runs one real TypeScript test against its native TSX app.
/// `cwd` anchors paths and `args` contains optional app, test file, and output.
///
/// # Errors
/// Returns an actionable build, host, assertion, or report error.
pub(super) fn test(cwd: &Path, args: &[String]) -> Result<(), String> {
    let (positionals, out) = parse_out(args)?;
    let (app, test) = match positionals.as_slice() {
        [test] => (cwd.to_path_buf(), resolve_test(cwd, cwd, test)),
        [app, test] => {
            let app = cwd.join(app);
            let test = resolve_test(cwd, &app, test);
            (app, test)
        }
        _ => {
            return Err(
                "usage: argui test [app-path] <file.test.ts|file.test.tsx> --out <directory>"
                    .into(),
            );
        }
    };
    if !test.to_string_lossy().ends_with(".test.ts")
        && !test.to_string_lossy().ends_with(".test.tsx")
    {
        return Err("test file must end in .test.ts or .test.tsx".into());
    }
    if !test.is_file() {
        return Err(format!("test file missing: {}", test.display()));
    }
    run(&app, &test, &cwd.join(out))
}

/// Takes a one-frame PNG of the real app through the same test driver.
/// `cwd` anchors app and output paths; `args` selects an optional app and PNG.
///
/// # Errors
/// Returns an actionable build, host, or screenshot error.
pub(super) fn screenshot(cwd: &Path, args: &[String]) -> Result<(), String> {
    let (positionals, out) = parse_out(args)?;
    let app = match positionals.as_slice() {
        [] => cwd.to_path_buf(),
        [app] => cwd.join(app),
        _ => return Err("usage: argui screenshot [app-path] --out <file.png>".into()),
    };
    let target = cwd.join(out);
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("screenshot needs a PNG filename")?;
    if target.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("screenshot output must end in .png".into());
    }
    let out = target
        .parent()
        .ok_or("screenshot needs a parent directory")?;
    fs::create_dir_all(out).map_err(|error| format!("{}: {error}", out.display()))?;
    let source = app
        .join("src/main.tsx")
        .canonicalize()
        .map_err(|error| format!("{}: {error}", app.join("src/main.tsx").display()))?;
    let module = serde_json::to_string(&source.to_string_lossy().as_ref())
        .map_err(|error| error.to_string())?;
    let script = format!(
        "import {{ mountGallery }} from {module}\nimport {{ defineArguiTest }} from '@argui/test'\nexport default defineArguiTest({{ app: mountGallery, async run(ui) {{ await ui.screenshot({name:?}) }} }})\n"
    );
    let test = out.join(".argui-screenshot.test.ts");
    fs::write(&test, script).map_err(|error| format!("{}: {error}", test.display()))?;
    let result = run(&app, &test, out);
    let _ = fs::remove_file(&test);
    result
}

/// Extracts the required `--out` value and remaining positional arguments.
///
/// # Errors
/// Returns a usage message for missing or repeated output switches.
fn parse_out(args: &[String]) -> Result<(Vec<&str>, &str), String> {
    let mut positionals = Vec::new();
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--out" {
            if output.is_some() {
                return Err("--out may appear only once".into());
            }
            output = args.get(index + 1).map(String::as_str);
            index += 2;
        } else {
            positionals.push(args[index].as_str());
            index += 1;
        }
    }
    let out = output
        .filter(|value| !value.is_empty())
        .ok_or("--out <path> is required")?;
    Ok((positionals, out))
}

/// Resolves a test path from the caller or selected app directory.
/// `cwd` is the caller directory and `app` the app directory.
fn resolve_test(cwd: &Path, app: &Path, test: &str) -> PathBuf {
    let caller = cwd.join(test);
    if caller.is_file() {
        caller
    } else {
        app.join(test)
    }
}

/// Builds the native host and test module, then samples the host until exit.
/// `app`, `test`, and `out` identify source and artifact paths.
///
/// # Errors
/// Returns a build, spawn, failed-test, or report error.
fn run(app: &Path, test: &Path, out: &Path) -> Result<(), String> {
    fs::create_dir_all(out).map_err(|error| format!("{}: {error}", out.display()))?;
    let binary = standalone::automation_binary(app)?;
    let bundle_dir = out.join(".argui-bundle");
    fs::create_dir_all(&bundle_dir).map_err(|error| error.to_string())?;
    let script = app.join("node_modules/.argui-build-test.mjs");
    fs::write(&script, include_str!("../assets/templates/build-test.mjs"))
        .map_err(|error| format!("{}: {error}", script.display()))?;
    let build = Command::new("bun")
        .arg(&script)
        .arg(app)
        .arg(test)
        .arg(&bundle_dir)
        .current_dir(app)
        .status()
        .map_err(|error| format!("Bun could not build the test: {error}"))?;
    let _ = fs::remove_file(&script);
    if !build.success() {
        return Err(format!("test bundle build failed with {build}"));
    }
    let mut command = Command::new(&binary);
    command
        .env("ARGUI_AUTOMATION_TEST", bundle_dir.join("test.mjs"))
        .env("ARGUI_AUTOMATION_OUT", out)
        .env("ARGUI_AUTOMATION_GATE", "1")
        .current_dir(app)
        .stdin(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("{}: {error}", binary.display()))?;
    let sampler_started = unix_ms();
    let sampler = ProcessSampler::start(child.id(), Duration::from_millis(250));
    std::thread::sleep(Duration::from_millis(260));
    child
        .stdin
        .take()
        .ok_or("host start gate unavailable")?
        .write_all(&[1])
        .map_err(|error| format!("host start gate: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("native test host: {error}"))?;
    let samples = sampler.finish();
    let report_path = out.join("report.json");
    let report = fs::read(&report_path)
        .ok()
        .and_then(|data| serde_json::from_slice::<Value>(&data).ok());
    let mut report = report.unwrap_or_else(|| {
        json!({ "ok": false,
        "error": format!("native test host exited with {status} before writing its report") })
    });
    let process = process_report(child.id(), sampler_started, &report, samples);
    report["processMetrics"] = process;
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("{}: {error}", report_path.display()))?;
    if status.success() && report["ok"] == true {
        println!("Argui test passed: {}", report_path.display());
        Ok(())
    } else {
        Err(format!(
            "Argui test failed: {}; report: {}",
            report["error"],
            report_path.display()
        ))
    }
}

/// Returns wall-clock milliseconds for aligning host and monitor timebases.
fn unix_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.0
}

/// Summarizes process samples and labels startup, interaction, idle, and teardown.
/// `root_pid` identifies the app host; `sampler_started` aligns timestamps;
/// `report` supplies host step times; `samples` carry cross-platform counters.
fn process_report(
    root_pid: u32,
    sampler_started: f64,
    report: &Value,
    samples: Vec<ProcessSample>,
) -> Value {
    let host_offset = report["startedUnixMs"]
        .as_f64()
        .map(|host| host - sampler_started)
        .unwrap_or(0.0);
    let steps = report["steps"].as_array().cloned().unwrap_or_default();
    let first = steps
        .first()
        .and_then(|step| step["started_ms"].as_f64())
        .unwrap_or(f64::INFINITY);
    let last = steps
        .last()
        .and_then(|step| Some(step["started_ms"].as_f64()? + step["duration_ms"].as_f64()?))
        .unwrap_or(first);
    let mut peak = None::<u64>;
    let mut children = Vec::<u32>::new();
    let rows = samples
        .iter()
        .map(|sample| {
            if sample.pid != root_pid && !children.contains(&sample.pid) {
                children.push(sample.pid);
            }
            if let Some(bytes) = sample.rss_bytes {
                peak = Some(peak.map_or(bytes, |old| old.max(bytes)));
            }
            let relative = sample.timestamp_ms - host_offset;
            let phase = if relative < first {
                "startup"
            } else if relative > last {
                "teardown"
            } else if steps.iter().any(|step| {
                let begin = step["started_ms"].as_f64().unwrap_or(f64::INFINITY);
                let end = begin + step["duration_ms"].as_f64().unwrap_or(0.0);
                relative >= begin && relative <= end
            }) {
                "interaction"
            } else {
                "idle"
            };
            json!({ "timestampMs": sample.timestamp_ms,
            "pid": sample.pid, "parentPid": sample.parent_pid,
            "cpuPercent": sample.cpu_percent, "rssBytes": sample.rss_bytes,
            "phase": phase })
        })
        .collect::<Vec<_>>();
    let unavailable = if rows.is_empty() {
        Some("process exited before the sampler could read OS counters")
    } else if rows.iter().all(|row| row["cpuPercent"].is_null()) {
        Some("process exited before a second CPU sample was available")
    } else {
        None
    };
    json!({ "hostPid": root_pid, "childPids": children,
        "samplerStartedUnixMs": sampler_started,
        "cpuUnits": "100% = one fully used logical core",
        "samples": rows, "peakRssBytes": peak,
        "unavailable": unavailable })
}
