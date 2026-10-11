//! The top-level error: everything the CLI can report, and the only error it prints.

use std::fmt::{self, Display, Write};
use std::iter::successors;

use crate::{cli, payload, signing};

/// Why nothing was signed.
///
/// Display shows the whole cause chain on one line, separated by `: `, with control characters
/// escaped: messages can quote untrusted payload text. Quotes and backslashes are kept, so
/// values already quoted with `{:?}` stay readable. Because Display already shows every cause,
/// `source()` returns `None`.
#[derive(Debug)]
pub enum Error {
    Cli(cli::Error),
    Payload(payload::Error),
    Signing(signing::Error),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner: &(dyn std::error::Error + 'static) = match self {
            Self::Cli(error) => error,
            Self::Payload(error) => error,
            Self::Signing(error) => error,
        };
        write_escaped(formatter, inner)?;
        for source in successors(inner.source(), |&error| error.source()) {
            formatter.write_str(": ")?;
            write_escaped(formatter, source)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<cli::Error> for Error {
    fn from(error: cli::Error) -> Self {
        Self::Cli(error)
    }
}

impl From<payload::Error> for Error {
    fn from(error: payload::Error) -> Self {
        Self::Payload(error)
    }
}

impl From<signing::Error> for Error {
    fn from(error: signing::Error) -> Self {
        Self::Signing(error)
    }
}

fn write_escaped(
    formatter: &mut fmt::Formatter<'_>,
    message: &(impl Display + ?Sized),
) -> fmt::Result {
    for c in message.to_string().chars() {
        match c {
            '"' | '\'' | '\\' => formatter.write_char(c)?,
            c => write!(formatter, "{}", c.escape_debug())?,
        }
    }
    Ok(())
}
