//! # Golden-file test harness
//!
//! Runs every script under `tests/cases/` through the real `rlox-lang` binary and compares its
//! **stdout** against a sibling `.expected` file. Stderr is ignored, except for showing the panic
//! message when the interpreter crashes.
//!
//! Case types (decided by the file extension):
//! - `foo.rlox`: run as `rlox-lang foo.rlox` (file mode)
//! - `foo.repl`: the file's contents are piped into the REPL via stdin (REPL mode), one line per prompt
//!
//! Each case `foo.<ext>` needs a `foo.expected` next to it. Subdirectories are allowed and show up
//! in the test name (e.g. `scanner/identifiers`).
//!
//! A case fails if:
//! - stdout doesn't match `.expected` (CRLF is normalised and trailing whitespace at the end is ignored)
//! - the interpreter crashes (panic / stack overflow / killed by a signal)
//! - it runs longer than the timeout (it is killed, so infinite loops don't hang the suite)
//! - `.expected` is missing
//!
//! ## Usage
//! ```text
//! cargo test                          # run everything
//! cargo test --test lox -- scanner    # only cases whose name contains "scanner" (any number of filters)
//! RLOX_BLESS=1 cargo test --test lox  # write/overwrite .expected files from the actual output
//! RLOX_TIMEOUT=2 cargo test           # per-case timeout in seconds (default 5)
//! ```
//! Blessing never records the output of a crashed or timed-out run. Always review blessed files
//! before committing: they record what the interpreter *does*, not what it *should* do.

use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const BIN: &str = env!("CARGO_BIN_EXE_rlox-lang");
const DEFAULT_TIMEOUT_SECS: f64 = 5.0;

struct Case {
    name: String,
    script: PathBuf,
    expected: PathBuf,
    repl: bool,
}

enum Outcome {
    Pass,
    Blessed,
    Fail(String),
}

struct RunResult {
    stdout: String,
    stderr: String,
    status: Option<ExitStatus>, // None => timed out
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("cases");
    // libtest-style flags (e.g. --nocapture) may be passed through by cargo; ignore them.
    let filters: Vec<String> = env::args().skip(1).filter(|a| !a.starts_with('-')).collect();
    let bless = env::var_os("RLOX_BLESS").is_some_and(|v| v != "0");
    let timeout = env::var("RLOX_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    let timeout = Duration::from_secs_f64(timeout);

    let mut cases = Vec::new();
    if root.is_dir() {
        collect_cases(&root, &root, &mut cases);
    }
    cases.sort_by(|a, b| a.name.cmp(&b.name));
    let total = cases.len();
    cases.retain(|c| filters.is_empty() || filters.iter().any(|f| c.name.contains(f.as_str())));
    let filtered_out = total - cases.len();

    println!("\nrunning {} rlox cases", cases.len());

    let mut failures: Vec<(String, String)> = Vec::new();
    let (mut passed, mut blessed) = (0, 0);

    for case in &cases {
        let outcome = run_case(case, bless, timeout);
        let label = match &outcome {
            Outcome::Pass => "ok",
            Outcome::Blessed => "BLESSED",
            Outcome::Fail(_) => "FAILED",
        };
        println!("test {} ... {label}", case.name);
        match outcome {
            Outcome::Pass => passed += 1,
            Outcome::Blessed => blessed += 1,
            Outcome::Fail(msg) => failures.push((case.name.clone(), msg)),
        }
    }

    if !failures.is_empty() {
        println!("\nfailures:");
        for (name, msg) in &failures {
            println!("\n---- {name} ----\n{msg}");
        }
        println!("\nfailures:");
        for (name, _) in &failures {
            println!("    {name}");
        }
    }

    let result = if failures.is_empty() { "ok" } else { "FAILED" };
    print!(
        "\ntest result: {result}. {passed} passed; {} failed; {filtered_out} filtered out",
        failures.len()
    );
    if bless {
        print!("; {blessed} blessed");
    }
    println!("\n");

    if total == 0 {
        println!("note: no cases found in {}", root.display());
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
}

fn collect_cases(root: &Path, dir: &Path, out: &mut Vec<Case>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("bad directory entry").path();
        if path.is_dir() {
            collect_cases(root, &path, out);
            continue;
        }
        let repl = match path.extension().and_then(|e| e.to_str()) {
            Some("rlox") => false,
            Some("repl") => true,
            _ => continue,
        };
        let name = path
            .strip_prefix(root)
            .unwrap()
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");
        out.push(Case {
            name,
            expected: path.with_extension("expected"),
            script: path,
            repl,
        });
    }
}

fn run_case(case: &Case, bless: bool, timeout: Duration) -> Outcome {
    let run = match execute(case, timeout) {
        Ok(run) => run,
        Err(e) => return Outcome::Fail(format!("could not run interpreter: {e}")),
    };

    // Crashes and timeouts are failures regardless of output, and are never blessed.
    let Some(status) = run.status else {
        return Outcome::Fail(format!(
            "timed out after {:.1}s (infinite loop?) and was killed\n\nstdout so far:\n{}",
            timeout.as_secs_f64(),
            indent(&run.stdout)
        ));
    };
    if let Some(reason) = crash_reason(&status) {
        return Outcome::Fail(format!(
            "interpreter crashed ({reason})\n\n{}\n\nstdout:\n{}",
            panic_excerpt(&run.stderr),
            indent(&run.stdout)
        ));
    }

    let actual = normalise(&run.stdout);
    let expected = fs::read_to_string(&case.expected).ok().map(|s| normalise(&s));

    match expected {
        Some(exp) if exp == actual => Outcome::Pass,
        _ if bless => {
            fs::write(&case.expected, format!("{actual}\n"))
                .unwrap_or_else(|e| panic!("cannot write {}: {e}", case.expected.display()));
            Outcome::Blessed
        }
        None => Outcome::Fail(format!(
            "missing {}\nactual output:\n{}\n(re-run with RLOX_BLESS=1 to record it)",
            case.expected.display(),
            indent(&actual)
        )),
        Some(exp) => Outcome::Fail(format!(
            "stdout did not match {}\n(- expected, + actual)\n{}",
            case.expected.display(),
            diff(&exp, &actual)
        )),
    }
}

fn execute(case: &Case, timeout: Duration) -> std::io::Result<RunResult> {
    let mut cmd = Command::new(BIN);
    if case.repl {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.arg(&case.script).stdin(Stdio::null());
    }
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    // Feed stdin and drain both pipes on separate threads so a chatty child (e.g. the `dbg!` token
    // dump on stderr) can never fill a pipe buffer and deadlock against us.
    let stdin_writer = child.stdin.take().map(|mut stdin| {
        let input = fs::read(&case.script).unwrap_or_default();
        thread::spawn(move || {
            let _ = stdin.write_all(&input); // child may exit early; ignore broken pipe
        })
    });
    let stdout = drain(child.stdout.take().unwrap());
    let stderr = drain(child.stderr.take().unwrap());

    let status = wait_with_timeout(&mut child, timeout)?;
    if let Some(h) = stdin_writer {
        let _ = h.join();
    }

    Ok(RunResult {
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
        status,
    })
}

fn drain<R: Read + Send + 'static>(mut pipe: R) -> JoinHandle<String> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        String::from_utf8_lossy(&buf).into_owned()
    })
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> std::io::Result<Option<ExitStatus>> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

/// `main` reports Lox errors with `println!` and exits 0, so any non-zero exit is a crash.
fn crash_reason(status: &ExitStatus) -> Option<String> {
    match status.code() {
        Some(0) => None,
        Some(101) => Some("panic, exit code 101".into()),
        Some(code) => Some(format!("exit code {code}")),
        None => Some(signal_description(status)),
    }
}

#[cfg(unix)]
fn signal_description(status: &ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    match status.signal() {
        Some(6) => "aborted by SIGABRT, e.g. stack overflow".into(),
        Some(11) => "SIGSEGV".into(),
        Some(sig) => format!("killed by signal {sig}"),
        None => "terminated abnormally".into(),
    }
}

#[cfg(not(unix))]
fn signal_description(_: &ExitStatus) -> String {
    "terminated abnormally".into()
}

/// Pulls the panic / abort message out of stderr, skipping the `dbg!` token dump and backtrace frames.
fn panic_excerpt(stderr: &str) -> String {
    let lines: Vec<&str> = stderr.lines().collect();
    let start = lines.iter().position(|l| {
        l.contains("panicked at") || l.contains("has overflowed its stack") || l.contains("fatal runtime error")
    });
    let excerpt: Vec<&str> = match start {
        Some(i) => lines[i..]
            .iter()
            .take_while(|l| !l.starts_with("stack backtrace:"))
            .take(6)
            .copied()
            .collect(),
        None => lines.iter().rev().take(10).rev().copied().collect(),
    };
    format!("stderr:\n{}", indent(&excerpt.join("\n")))
}

fn normalise(s: &str) -> String {
    s.replace("\r\n", "\n").trim_end().to_string()
}

fn indent(s: &str) -> String {
    if s.is_empty() {
        return "    <empty>".into();
    }
    s.lines().map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n")
}

/// Minimal line diff (LCS). Outputs are small, so O(n*m) is fine.
fn diff(expected: &str, actual: &str) -> String {
    let a: Vec<&str> = expected.lines().collect();
    let b: Vec<&str> = actual.lines().collect();
    let (n, m) = (a.len(), b.len());

    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            out.push(format!("      {}", a[i]));
            i += 1;
            j += 1;
        } else if i < n && (j == m || lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.push(format!("    - {}", a[i]));
            i += 1;
        } else {
            out.push(format!("    + {}", b[j]));
            j += 1;
        }
    }
    out.join("\n")
}
