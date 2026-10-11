// Marks helpers as test code so clippy.toml's allow-unwrap-in-tests covers them.
#![cfg(test)]

//! The command-line flow in process with scripted terminal input, then the built binary with
//! and without a pseudo-terminal. Never fund these keys.

// Each test binary uses only some of the shared helpers.
#[allow(dead_code)]
mod common;

use std::error::Error as _;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use bitcoin::consensus::encode::serialize_hex;
use bitcoin::{NetworkKind, PrivateKey, base58};
use rustix::fs::{Mode, OFlags};
use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};
use rustix::termios::{LocalModes, tcgetattr};
use serde_json::json;
use zeroize::Zeroizing;

use btc_offline_signer::Error;
use btc_offline_signer::cli::{self, run};
use btc_offline_signer::payload::{self, ConstraintError};
use btc_offline_signer::signing::{self, sign};

use crate::common::{
    KEY_1_ADDRESS, KEY_1_WIF, KEY_2_WIF, RECIPIENT_ADDRESS, fixture, fixture_with,
    fixture_with_change, parse_ok,
};

const SUMMARY: &str = "\
Spend   100000 sats from 71e3be4e5b89d4263d5902a50180ea5b660a3af3ee8894aaa024ec0f2391a0f6:0
Pay     60000 sats to tb1qcr8te4kr609gcawutmrza0j4xv80jy8zmfp6l0
Change  35000 sats to tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx
Fee     5000 sats
Type yes to sign: ";
const KEY_PROMPT: &str = "Testnet WIF private key: ";

/// Writes `params` to a file named after the test.
fn params_file(name: &str, params: impl Display) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.json"));
    fs::write(&path, params.to_string()).unwrap();
    path
}

/// Runs the change fixture with `reply` and `wif` typed, returning the result and the terminal
/// text.
fn run_with(reply: &str, wif: &str) -> (Result<String, Error>, String) {
    let params = params_file(&format!("run-{reply:?}-{wif:?}"), fixture_with_change());
    let mut terminal = Vec::new();
    let result = run(
        &params,
        &mut terminal,
        || Ok(reply.to_owned()),
        || Ok(Zeroizing::new(wif.to_owned())),
    );
    (result, String::from_utf8(terminal).unwrap())
}

fn key_error(wif: &str) -> Error {
    run_with("yes\n", wif).0.unwrap_err()
}

fn wif_with(edit: impl FnOnce(&mut PrivateKey)) -> String {
    let mut key = PrivateKey::from_wif(KEY_1_WIF).unwrap();
    edit(&mut key);
    key.to_wif()
}

#[test]
fn signs_after_confirmation_and_shows_payment() {
    let expected = sign(
        &parse_ok(fixture_with_change()),
        &PrivateKey::from_wif(KEY_1_WIF).unwrap(),
    )
    .unwrap();

    for reply in ["yes\n", "yes\r\n"] {
        let (hex, terminal) = run_with(reply, KEY_1_WIF);
        assert_eq!(hex.unwrap(), serialize_hex(&expected));
        assert_eq!(terminal, SUMMARY);
        assert!(terminal.contains(RECIPIENT_ADDRESS) && terminal.contains(KEY_1_ADDRESS));
    }
}

#[test]
fn returns_open_params_for_missing_file() {
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("missing.json");
    let error = run(
        &missing,
        &mut io::sink(),
        || unreachable!(),
        || unreachable!(),
    )
    .unwrap_err();

    assert!(matches!(
        &error,
        Error::Cli(cli::Error::OpenParams { path, source }) if *path == missing && source.kind() == io::ErrorKind::NotFound
    ));
    // Display already shows the cause, so it is not also a source.
    assert!(error.source().is_none());
}

#[test]
fn returns_payload_for_invalid_params_before_prompting() {
    let params = params_file(
        "invalid-network",
        fixture_with("/network", json!("bitcoin")),
    );
    let mut terminal = Vec::new();
    let error = run(&params, &mut terminal, || unreachable!(), || unreachable!()).unwrap_err();

    assert!(matches!(
        error,
        Error::Payload(payload::Error::Constraints(
            ConstraintError::UnsupportedNetwork(_)
        ))
    ));
    assert!(terminal.is_empty());
}

#[test]
fn returns_terminal_for_read_failure() {
    let params = params_file("terminal-failure", fixture());
    // A forged window title and a fake second line.
    let hostile = "\u{1b}]0;FORGED\u{7}\r\nerror: forged line";
    let failed = || io::Error::other(hostile);

    for result in [
        run(
            &params,
            &mut io::sink(),
            || Err(failed()),
            || unreachable!(),
        ),
        run(
            &params,
            &mut io::sink(),
            || Ok("yes\n".to_owned()),
            || Err(failed()),
        ),
    ] {
        let error = result.unwrap_err();
        assert!(
            matches!(&error, Error::Cli(cli::Error::Terminal(source)) if source.to_string() == hostile)
        );
        assert_eq!(
            error.to_string(),
            r"terminal I/O failed: \u{1b}]0;FORGED\u{7}\r\nerror: forged line"
        );
    }
}

#[test]
fn returns_cancelled_for_reply_other_than_yes() {
    let params = params_file("cancelled", fixture());
    // An empty reply is EOF; "yes" without a newline is EOF after typing it.
    for reply in ["", "\n", "no\n", "y\n", "YES\n", "yes \n", " yes\n", "yes"] {
        let mut key_requested = false;
        let result = run(
            &params,
            &mut io::sink(),
            || Ok(reply.to_owned()),
            || {
                key_requested = true;
                Ok(Zeroizing::new(KEY_1_WIF.to_owned()))
            },
        );
        assert!(
            matches!(result, Err(Error::Cli(cli::Error::Cancelled))),
            "{reply:?}"
        );
        assert!(!key_requested, "{reply:?}");
    }
}

#[test]
fn returns_cancelled_for_interrupted_key_prompt() {
    let params = params_file("interrupted", fixture());
    let result = run(
        &params,
        &mut io::sink(),
        || Ok("yes\n".to_owned()),
        || Err(io::ErrorKind::Interrupted.into()),
    );
    assert!(matches!(result, Err(Error::Cli(cli::Error::Cancelled))));
}

#[test]
fn returns_invalid_key_for_malformed_wif() {
    let bad_checksum = KEY_1_WIF.replace('A', "B");
    // Base58 errors would quote '0' or the checksum; the message must not.
    for wif in [
        "",
        "not a key",
        &KEY_1_WIF.replace('A', "0"),
        &bad_checksum,
        // A valid base58check testnet address is not a key.
        "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn",
    ] {
        let (result, terminal) = run_with("yes\n", wif);
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Cli(cli::Error::InvalidKey)),
            "{wif:?}"
        );
        assert_eq!(error.to_string(), "private key is not a valid WIF");
        assert_eq!(terminal, SUMMARY);
    }
}

#[test]
fn returns_invalid_key_for_noncanonical_compression_marker() {
    // Testnet version byte, secret 1, then the compression marker, which must be 0x01.
    let wif = |marker| {
        let mut payload = vec![0xef];
        payload.extend([0; 31]);
        payload.extend([1, marker]);
        base58::encode_check(&payload)
    };
    assert_eq!(wif(0x01), KEY_1_WIF);

    // The checksum is valid, and `PrivateKey::from_wif` alone accepts it.
    assert!(matches!(
        key_error(&wif(0x02)),
        Error::Cli(cli::Error::InvalidKey)
    ));
}

#[test]
fn returns_signing_for_mainnet_key() {
    let mainnet = wif_with(|key| key.network = NetworkKind::Main);
    let error = key_error(&mainnet);
    assert!(matches!(error, Error::Signing(signing::Error::MainnetKey)));
    assert_eq!(
        error.to_string(),
        "private key is for mainnet; only testnet keys are supported"
    );
}

#[test]
fn returns_signing_for_uncompressed_key() {
    let uncompressed = wif_with(|key| key.compressed = false);
    let error = key_error(&uncompressed);
    assert!(matches!(
        error,
        Error::Signing(signing::Error::UncompressedKey)
    ));
    assert_eq!(
        error.to_string(),
        "private key is uncompressed; P2WPKH requires a compressed key"
    );
}

#[test]
fn returns_signing_for_key_mismatch() {
    let error = key_error(KEY_2_WIF);
    assert!(matches!(error, Error::Signing(signing::Error::KeyMismatch)));
    assert_eq!(
        error.to_string(),
        "private key does not control the previous output"
    );
}

#[test]
fn display_escapes_control_characters_from_params() {
    let mut params = fixture();
    params["\u{1b}[2J"] = json!(null);
    let params = params_file("control-characters", params);
    let error = run(
        &params,
        &mut io::sink(),
        || unreachable!(),
        || unreachable!(),
    )
    .unwrap_err();

    let report = error.to_string();
    assert!(!report.contains('\u{1b}'), "{report:?}");
    assert!(
        report.starts_with(r"invalid params JSON: unknown field `\u{1b}[2J`"),
        "{report}"
    );
}

/// Runs the built binary with no terminal on stdin.
fn signer(args: &[impl AsRef<OsStr>]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_btc-offline-signer"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

#[test]
fn binary_rejects_bad_arguments_with_empty_stdout() {
    let params = params_file("binary-arguments", fixture());
    let params = params.to_str().unwrap();

    for args in [
        &[][..],
        &["--params"],
        &["--params", params, "--key", KEY_1_WIF],
        &[params],
    ] {
        let output = signer(args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(stderr(&output).starts_with("error: "), "{args:?}");
        // Rejected arguments are not echoed back.
        assert!(!stderr(&output).contains(KEY_1_WIF), "{args:?}");
    }

    let help = signer(&["--help"]);
    assert!(help.status.success());
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("--params <PATH>")
    );
}

#[test]
fn binary_reports_failures_on_stderr_with_empty_stdout() {
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("binary-missing.json");
    let invalid = params_file("binary-invalid", fixture_with("/network", json!("bitcoin")));
    let prev_tx = fixture()["inputs"][0]["prev_tx"]
        .as_str()
        .unwrap()
        .to_owned();
    let trailing = fixture_with("/inputs/0/prev_tx", json!(format!("{prev_tx}00")));
    let trailing = params_file("binary-trailing-bytes", trailing);
    let valid = params_file("binary-valid", fixture_with_change());

    for (params, message) in [
        (
            &missing,
            format!(
                "error: failed to open params file {missing:?}: No such file or directory (os error 2)\n"
            ),
        ),
        (
            &invalid,
            "error: unsupported network \"bitcoin\"; only testnet is supported\n".to_owned(),
        ),
        // Nested errors show each cause once.
        (
            &trailing,
            "error: prev_tx is not a valid transaction: decoding error: attempted to decode \
             object from an iterator that yielded too many bytes\n"
                .to_owned(),
        ),
        // Piped stdin cannot confirm, so the key is never requested.
        (
            &valid,
            format!("{SUMMARY}error: terminal I/O failed: stdin is not a terminal\n"),
        ),
    ] {
        let output = signer(&["--params", params.to_str().unwrap()]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(stderr(&output), message);
    }
}

#[test]
fn binary_escapes_hostile_params_path() {
    use std::os::unix::ffi::OsStrExt;

    // ESC, CR, LF, and a byte that is not UTF-8.
    let name = OsStr::from_bytes(b"missing-\x1b[2J\r\n\xff.json");
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let output = signer(&[OsStr::new("--params"), path.as_os_str()]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    // One line: the newline in the path is escaped too.
    let expected = format!(
        r#"error: failed to open params file "{dir}/missing-\u{{1b}}[2J\r\n\xFF.json": No such file or directory (os error 2)"#,
        dir = env!("CARGO_TARGET_TMPDIR"),
    );
    assert_eq!(stderr(&output), format!("{expected}\n"));
}

/// A new pseudo-terminal that never becomes this process's controlling terminal.
fn open_pty() -> (File, File) {
    let master = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).unwrap();
    grantpt(&master).unwrap();
    unlockpt(&master).unwrap();
    let name = ptsname(&master, Vec::new()).unwrap();
    let flags = OFlags::RDWR | OFlags::NOCTTY | OFlags::CLOEXEC;
    let slave = rustix::fs::open(name.as_c_str(), flags, Mode::empty()).unwrap();
    (File::from(master), File::from(slave))
}

struct TerminalRun {
    status: ExitStatus,
    /// Everything the terminal showed, including echoed input, with `\r\n` line endings.
    shown: String,
    /// The terminal's local modes before the run and after the binary exited.
    modes: [LocalModes; 2],
}

/// Runs the binary on the change fixture with stdin and stderr on a new pseudo-terminal,
/// typing each input once the text before it appears.
fn sign_on_terminal(name: &str, stdout: File, typed: &[(&str, &[u8])]) -> TerminalRun {
    let params = params_file(name, fixture_with_change());
    let (master, slave) = open_pty();
    let before = tcgetattr(&slave).unwrap().local_modes;
    let mut child = Command::new(env!("CARGO_BIN_EXE_btc-offline-signer"))
        .arg("--params")
        .arg(&params)
        .stdin(slave.try_clone().unwrap())
        .stderr(slave.try_clone().unwrap())
        .stdout(stdout)
        .spawn()
        .unwrap();

    // Reads block until the binary writes, so they run on their own thread.
    let (sender, received) = mpsc::channel();
    let mut screen = master.try_clone().unwrap();
    thread::spawn(move || {
        let mut chunk = [0; 1024];
        while let Ok(len @ 1..) = screen.read(&mut chunk) {
            if sender.send(chunk[..len].to_vec()).is_err() {
                break;
            }
        }
    });
    let mut shown = Vec::new();
    let mut keyboard = &master;
    for (prompt, input) in typed {
        while !String::from_utf8_lossy(&shown).contains(prompt) {
            let chunk = received.recv_timeout(Duration::from_secs(10));
            shown.extend(chunk.expect("prompt not shown"));
        }
        keyboard.write_all(input).unwrap();
    }
    // A binary still waiting for input would otherwise block the test forever.
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if started.elapsed() > Duration::from_secs(10) {
            child.kill().unwrap();
            break None;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let status = status.expect("binary did not exit, so it was killed");
    // The binary has exited, so its remaining output is already in the terminal.
    while let Ok(chunk) = received.recv_timeout(Duration::from_millis(200)) {
        shown.extend(chunk);
    }

    TerminalRun {
        status,
        shown: String::from_utf8(shown).unwrap(),
        modes: [before, tcgetattr(&slave).unwrap().local_modes],
    }
}

#[test]
fn binary_signs_on_terminal_without_echoing_key() {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("terminal-signed.hex");
    let run = sign_on_terminal(
        "terminal-signed",
        File::create(&path).unwrap(),
        &[
            ("Type yes to sign: ", b"yes\n"),
            (KEY_PROMPT, format!("{KEY_1_WIF}\r").as_bytes()),
        ],
    );
    let expected = sign(
        &parse_ok(fixture_with_change()),
        &PrivateKey::from_wif(KEY_1_WIF).unwrap(),
    )
    .unwrap();

    assert!(run.status.success(), "{}", run.shown);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!("{}\n", serialize_hex(&expected))
    );
    assert!(
        run.shown.contains("Type yes to sign: yes\r\n"),
        "{}",
        run.shown
    );
    assert!(!run.shown.contains(KEY_1_WIF), "{}", run.shown);
    assert!(run.modes[0].contains(LocalModes::ECHO | LocalModes::ICANON | LocalModes::ISIG));
    assert_eq!(run.modes[1], run.modes[0]);
}

#[test]
fn binary_restores_terminal_after_ctrl_c_at_key_prompt() {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("terminal-cancelled.hex");
    let run = sign_on_terminal(
        "terminal-cancelled",
        File::create(&path).unwrap(),
        &[("Type yes to sign: ", b"yes\n"), (KEY_PROMPT, b"cMah\x03")],
    );

    assert_eq!(run.status.code(), Some(1));
    assert!(fs::read(&path).unwrap().is_empty());
    assert!(
        run.shown.ends_with("error: signing cancelled\r\n"),
        "{}",
        run.shown
    );
    assert!(!run.shown.contains("cMah"), "{}", run.shown);
    assert_eq!(run.modes[1], run.modes[0]);
}

/// Only failures before output begins leave stdout empty; this one exits nonzero after.
#[cfg(target_os = "linux")]
#[test]
fn binary_reports_failure_to_write_signed_transaction() {
    let full = File::options().write(true).open("/dev/full").unwrap();
    let run = sign_on_terminal(
        "terminal-full",
        full,
        &[
            ("Type yes to sign: ", b"yes\n"),
            (KEY_PROMPT, format!("{KEY_1_WIF}\r").as_bytes()),
        ],
    );

    assert_eq!(run.status.code(), Some(1));
    assert!(
        run.shown.ends_with(
            "error: failed to write the signed transaction: No space left on device (os error 28)\r\n"
        ),
        "{}",
        run.shown
    );
}
