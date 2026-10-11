// Marks helpers as test code so clippy.toml's allow-unwrap-in-tests covers them.
#![cfg(test)]

// Each test binary uses only some of the shared helpers.
#[allow(dead_code)]
mod common;

#[path = "payload/messages.rs"]
mod messages;
#[path = "payload/parsing.rs"]
mod parsing;
#[path = "payload/payments.rs"]
mod payments;
#[path = "payload/previous_transaction.rs"]
mod previous_transaction;
