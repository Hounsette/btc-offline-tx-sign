use std::io::{self, IsTerminal, Stdin, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use rustix::termios::{LocalModes, OptionalActions, SpecialCodeIndex, tcgetattr, tcsetattr};
use zeroize::Zeroizing;

use btc_offline_signer::{Error, cli};

/// WIF keys are 51 or 52 characters. Longer input is cut here, then fails validation.
const MAX_KEY_LEN: usize = 128;

/// Signs a one-input P2WPKH testnet transaction offline and prints its hex to stdout.
///
/// Shows the payment on stderr, asks for confirmation, then reads the private key through a
/// hidden terminal prompt.
#[derive(Parser)]
#[command(about)]
struct Args {
    /// JSON payload describing the payment
    #[arg(long, value_name = "PATH")]
    params: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match cli::run(&args.params, &mut io::stderr(), read_reply, read_key)
        .and_then(|hex| print(&hex))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&error),
    }
}

/// The only place errors reach the terminal. Taking [`Error`] keeps every message escaped.
#[expect(clippy::print_stderr, reason = "errors are printed only here")]
fn fail(error: &Error) -> ExitCode {
    eprintln!("error: {error}");
    ExitCode::FAILURE
}

/// A failed write can leave part of the hex behind; callers discard stdout on failure.
fn print(hex: &str) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{hex}")
        .and_then(|()| stdout.flush())
        .map_err(|error| cli::Error::WriteTransaction(error).into())
}

/// One line typed on the terminal; piped input cannot confirm.
fn read_reply() -> io::Result<String> {
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        return Err(io::Error::other("stdin is not a terminal"));
    }
    let mut reply = String::new();
    stdin.read_line(&mut reply)?;
    Ok(reply)
}

/// Prompts for the key with echo off. Ctrl-C or Ctrl-D cancels. The terminal settings are
/// restored before returning, whatever the outcome.
fn read_key() -> io::Result<Zeroizing<String>> {
    let stdin = io::stdin();
    let original = tcgetattr(&stdin)?;
    let mut hidden = original.clone();
    // Without ISIG, Ctrl-C arrives as a byte instead of ending the process before the restore.
    hidden
        .local_modes
        .remove(LocalModes::ECHO | LocalModes::ICANON | LocalModes::ISIG);
    hidden.special_codes[SpecialCodeIndex::VMIN] = 1;
    hidden.special_codes[SpecialCodeIndex::VTIME] = 0;
    tcsetattr(&stdin, OptionalActions::Now, &hidden)?;
    // Prompt only once echo is off, so nothing typed is shown.
    let key =
        write!(io::stderr(), "Testnet WIF private key: ").and_then(|()| read_hidden_line(&stdin));
    // Discard pasted input left after a newline or cancellation before the shell can read it.
    tcsetattr(&stdin, OptionalActions::Flush, &original)?;
    writeln!(io::stderr())?;
    key
}

fn read_hidden_line(stdin: &Stdin) -> io::Result<Zeroizing<String>> {
    // Fixed capacity: growing would leave copies behind that are never zeroized.
    let mut line = Zeroizing::new(String::with_capacity(MAX_KEY_LEN));
    let mut byte = [0];
    loop {
        // Unbuffered, so stdin's buffer keeps no copy of the key.
        if rustix::io::read(stdin, &mut byte)? == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        match byte[0] {
            b'\r' | b'\n' => return Ok(line),
            // Ctrl-C, Ctrl-D
            0x03 | 0x04 => return Err(io::ErrorKind::Interrupted.into()),
            // Backspace, Delete
            0x08 | 0x7f => {
                line.pop();
            }
            printable @ b' '..=b'~' if line.len() < MAX_KEY_LEN => {
                line.push(char::from(printable));
            }
            _ => {}
        }
    }
}
