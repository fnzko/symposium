//! Windows-only probes for where a timed-out plugin hook is still waited for.
//! Throwaway: this file is not meant to be merged.

use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::json;
use symposium::hook_schema::{HookAgent, agent_event};
use symposium_testlib::{HookStep, TestContext, TestMode, with_fixture};

const WATCHED: &[&str] = &[
    "sh.exe",
    "bash.exe",
    "sleep.exe",
    "cat.exe",
    "cargo-agents.exe",
    "taskkill.exe",
];

fn ms() -> u128 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis()
}

fn log(message: impl AsRef<str>) {
    eprintln!("[diag {:>6} ms] {}", ms(), message.as_ref());
}

#[derive(Clone, Debug)]
struct Proc {
    pid: u32,
    parent: u32,
    name: String,
}

#[cfg(not(windows))]
fn processes() -> Vec<Proc> {
    Vec::new()
}

#[cfg(windows)]
fn processes() -> Vec<Proc> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };

    let mut found = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        assert_ne!(snapshot, INVALID_HANDLE_VALUE);
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut more = Process32FirstW(snapshot, &mut entry);
        while more != 0 {
            let len = entry
                .szExeFile
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(entry.szExeFile.len());
            found.push(Proc {
                pid: entry.th32ProcessID,
                parent: entry.th32ParentProcessID,
                name: String::from_utf16_lossy(&entry.szExeFile[..len]),
            });
            more = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
    }
    found
}

fn snapshot(label: &str) -> Vec<Proc> {
    let all = processes();
    let alive: BTreeSet<u32> = all.iter().map(|process| process.pid).collect();
    log(format!("process snapshot: {label}"));
    let watched: Vec<Proc> = all
        .into_iter()
        .filter(|process| WATCHED.contains(&process.name.to_lowercase().as_str()))
        .collect();
    for process in &watched {
        let parent = if alive.contains(&process.parent) {
            "alive"
        } else {
            "GONE"
        };
        eprintln!(
            "                     {:<18} pid={:<6} parent={:<6} (parent {parent})",
            process.name, process.pid, process.parent
        );
    }
    if watched.is_empty() {
        eprintln!("                     (none of {WATCHED:?})");
    }
    watched
}

fn sleeps(processes: &[Proc]) -> BTreeSet<u32> {
    processes
        .iter()
        .filter(|process| process.name.eq_ignore_ascii_case("sleep.exe"))
        .map(|process| process.pid)
        .collect()
}

fn run_logged(program: &str, args: &[&str]) {
    log(format!("running `{program} {}`", args.join(" ")));
    match Command::new(program).args(args).output() {
        Ok(output) => {
            log(format!(
                "`{program}` returned, status {:?}",
                output.status.code()
            ));
            for stream in [&output.stdout, &output.stderr] {
                for line in String::from_utf8_lossy(stream).lines() {
                    if !line.trim().is_empty() {
                        eprintln!("                     | {line}");
                    }
                }
            }
        }
        Err(error) => log(format!("`{program}` did not start: {error}")),
    }
}

fn kill_pids(pids: &BTreeSet<u32>) {
    for pid in pids {
        run_logged("taskkill", &["/F", "/PID", &pid.to_string()]);
    }
}

#[test]
#[cfg_attr(not(windows), ignore)]
fn a_taskkill_tree_reach() {
    log(format!("test process pid={}", std::process::id()));
    for tool in ["sh", "sleep", "taskkill"] {
        run_logged("where", &[tool]);
    }
    let before = sleeps(&snapshot("probe 1, before spawning"));
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("hang.sh");
    std::fs::write(&script, "sleep 30\n").unwrap();
    let mut child = Command::new("sh")
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(tempfile::tempfile().unwrap())
        .stderr(tempfile::tempfile().unwrap())
        .spawn()
        .unwrap();
    let sh = child.id();
    log(format!("spawned `sh hang.sh` (sleep 30), sh pid={sh}"));
    std::thread::sleep(Duration::from_millis(1500));
    let started: BTreeSet<u32> = sleeps(&snapshot("probe 1, script running"))
        .difference(&before)
        .copied()
        .collect();
    log(format!("sleep.exe started by the script: {started:?}"));
    run_logged("taskkill", &["/F", "/T", "/PID", &sh.to_string()]);
    let _ = child.kill();
    log(format!("sh reaped: {:?}", child.wait()));
    std::thread::sleep(Duration::from_millis(500));
    let survivors: BTreeSet<u32> = sleeps(&snapshot("probe 1, after taskkill /T"))
        .intersection(&started)
        .copied()
        .collect();
    log(format!(
        "PROBE 1 RESULT: sleep.exe started by the script {started:?}, still running after `taskkill /F /T /PID {sh}`: {survivors:?}"
    ));
    kill_pids(&survivors);
}

fn spawn_hook(ctx: &TestContext, stdout: Stdio, stderr: Stdio) -> Child {
    let step = HookStep::PreToolUse {
        tool_name: "DenyProbe".to_string(),
        tool_input: json!({"command": "ls"}),
    };
    let cwd = ctx.tempdir.display().to_string();
    let payload = agent_event(HookAgent::Claude, step.event())
        .unwrap()
        .translate_input(&step.to_input_event(&cwd))
        .to_string()
        .unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_cargo-agents").expect("must run via cargo test");
    let mut child = Command::new(binary)
        .args(["hook", "claude", "pre-tool-use"])
        .env("SYMPOSIUM_HOME", ctx.sym.config_dir())
        .current_dir(&ctx.tempdir)
        .stdin(Stdio::piped())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .expect("spawn cargo-agents");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.as_bytes())
        .unwrap();
    child
}

fn wait_for_exit(child: &mut Child, probe: &str, spawned: Instant) -> Duration {
    let mut snapped = false;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            let after = spawned.elapsed();
            log(format!(
                "{probe}: cargo-agents exited {} ms after spawn, {status}",
                after.as_millis()
            ));
            return after;
        }
        if !snapped && spawned.elapsed() >= Duration::from_secs(2) {
            snapped = true;
            snapshot(&format!("{probe}, hook running, before the 5 s limit"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(windows), ignore)]
async fn b_hook_read_through_pipes() {
    with_fixture(TestMode::SimulationOnly, &["hook-timeout0"], async |ctx| {
        let before = sleeps(&snapshot("probe 2, before spawning"));
        let spawned = Instant::now();
        let mut child = spawn_hook(&ctx, Stdio::piped(), Stdio::piped());
        log(format!(
            "probe 2: spawned cargo-agents pid={}, its stdout and stderr are pipes",
            child.id()
        ));
        let (tx, rx) = mpsc::channel();
        let pipes: [(&str, Box<dyn Read + Send>); 2] = [
            ("stdout", Box::new(child.stdout.take().unwrap())),
            ("stderr", Box::new(child.stderr.take().unwrap())),
        ];
        for (name, mut pipe) in pipes {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let mut bytes = Vec::new();
                let _ = pipe.read_to_end(&mut bytes);
                let _ = tx.send((name, spawned.elapsed(), bytes));
            });
        }
        let exited = wait_for_exit(&mut child, "probe 2", spawned);
        let left: BTreeSet<u32> = sleeps(&snapshot("probe 2, cargo-agents exited"))
            .difference(&before)
            .copied()
            .collect();
        log(format!(
            "probe 2: sleep.exe still running after cargo-agents exited: {left:?}"
        ));

        let mut eofs = Vec::new();
        let receive = |wait: Duration, eofs: &mut Vec<String>| {
            let until = Instant::now() + wait;
            while eofs.len() < 2 {
                let Ok((name, after, bytes)) =
                    rx.recv_timeout(until.saturating_duration_since(Instant::now()))
                else {
                    break;
                };
                log(format!(
                    "probe 2: EOF on {name} {} ms after spawn, {} bytes: {}",
                    after.as_millis(),
                    bytes.len(),
                    String::from_utf8_lossy(&bytes).trim()
                ));
                eofs.push(format!("{name} EOF at {} ms", after.as_millis()));
            }
        };
        receive(Duration::from_secs(3), &mut eofs);
        let mut killed_to_unblock = false;
        if eofs.len() < 2 {
            killed_to_unblock = true;
            log(
                "probe 2: a pipe is still open 3 s after cargo-agents exited; killing the leftover sleep.exe to see whether it holds the pipe",
            );
            kill_pids(&left);
            receive(Duration::from_secs(40), &mut eofs);
        }
        log(format!(
            "PROBE 2 RESULT: cargo-agents exited at {} ms; {}; leftover sleep.exe {left:?}; EOF needed the leftover sleep.exe killed: {killed_to_unblock}",
            exited.as_millis(),
            eofs.join(", "),
        ));
        kill_pids(&left);
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(windows), ignore)]
async fn c_hook_output_to_files() {
    with_fixture(TestMode::SimulationOnly, &["hook-timeout0"], async |ctx| {
        let before = sleeps(&snapshot("probe 3, before spawning"));
        let spawned = Instant::now();
        let mut child = spawn_hook(
            &ctx,
            tempfile::tempfile()?.into(),
            tempfile::tempfile()?.into(),
        );
        log(format!(
            "probe 3: spawned cargo-agents pid={}, its stdout and stderr are files",
            child.id()
        ));
        let exited = wait_for_exit(&mut child, "probe 3", spawned);
        let left: BTreeSet<u32> = sleeps(&snapshot("probe 3, cargo-agents exited"))
            .difference(&before)
            .copied()
            .collect();
        log(format!(
            "PROBE 3 RESULT: cargo-agents exited at {} ms; leftover sleep.exe {left:?}",
            exited.as_millis()
        ));
        kill_pids(&left);
        Ok(())
    })
    .await
    .unwrap();
}
