#!/usr/bin/env python3
"""Runs the built signer on a controlling pseudo-terminal, as a terminal emulator would: real echo,
line editing, Ctrl-C signals, and terminal modes. Checks argument/file errors, cancellation, bad
keys, empty stdout on failure, hidden key input, and the successful flow. Never fund these keys.

Usage: python3 tests/cli_test.py   (Unix; builds the debug binary first)
"""

import fcntl
import json
import os
import select
import signal
import subprocess
import tempfile
import termios
import time
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / os.environ.get("CARGO_TARGET_DIR", "target") / "debug" / "btc-offline-signer"
PARAMS = ROOT / "tests" / "common" / "payload.json"
TIMEOUT = 10

# Secret 1 owns the fixture's previous output; secret 2 does not.
KEY_1_WIF = "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN87JcbXMTcA"
KEY_1_MAINNET_WIF = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn"
KEY_1_UNCOMPRESSED_WIF = "91avARGdfge8E4tZfYLoxeJ5sGBdNJQH4kvjJoQFacbgwmaKkrx"
KEY_2_WIF = "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN87K7XCyj5v"

SUMMARY = """\
Spend   100000 sats from 71e3be4e5b89d4263d5902a50180ea5b660a3af3ee8894aaa024ec0f2391a0f6:0
Pay     95000 sats to tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx
Fee     5000 sats
Type yes to sign: """
KEY_PROMPT = "Testnet WIF private key: "
# What the terminal shows once the key is entered: the hidden input adds nothing but a newline.
KEY_ENTERED = f"{SUMMARY}yes\n{KEY_PROMPT}\n"
# The fixture signed by secret 1. RFC 6979 signing is deterministic; signing_test.rs verifies it.
SIGNED_HEX = (
    "02000000000101f6a091230fec24a0aa9488eef33a0a665bea8001a502593d26d4895b4ebee3710000000000fdff"
    "ffff011873010000000000160014751e76e8199196d454941c45d1b3a323f1433bd602483045022100b89d227138"
    "324928e6f0b61ab7b51159cf97fd75b117fddbdb9e17f0cc7cf40c022033ef034f61acc8f80c92cfc8118405b0b6"
    "9c365e51fe32ae706caad91cd2976001210279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b"
    "16f8179800000000"
)


def setUpModule():
    subprocess.run(["cargo", "build", "--locked", "--quiet"], cwd=ROOT, check=True)


def run_piped(*args):
    """Runs the binary with stdin from /dev/null. Returns the exit code, stdout, and stderr."""
    done = subprocess.run(
        [BIN, *args], stdin=subprocess.DEVNULL, capture_output=True, timeout=TIMEOUT
    )
    return done.returncode, done.stdout, done.stderr.decode()


def read(fd, timeout):
    ready, _, _ = select.select([fd], [], [], timeout)
    return os.read(fd, 4096) if ready else b""


def run_on_terminal(*typed, params=PARAMS):
    """Runs the binary with stdin and stderr on a new controlling pseudo-terminal, typing each
    `(text, keys)` once `text` is shown. Returns the exit code (negative for a signal), stdout,
    everything the terminal showed with `\\n` line endings, and whether its modes were restored.
    """
    master, slave = os.openpty()
    try:
        before = termios.tcgetattr(slave)
        with subprocess.Popen(
            [BIN, "--params", params],
            stdin=slave,
            stderr=slave,
            stdout=subprocess.PIPE,
            # Its own session on this terminal, so Ctrl-C sends SIGINT as in a real one.
            start_new_session=True,
            preexec_fn=lambda: fcntl.ioctl(0, termios.TIOCSCTTY, 0),
        ) as child:
            shown = b""
            try:
                for text, keys in typed:
                    deadline = time.monotonic() + TIMEOUT
                    while text.encode() not in shown:
                        assert time.monotonic() < deadline, f"{text!r} not shown in {shown!r}"
                        shown += read(master, 0.1)
                    os.write(master, keys)
                stdout, _ = child.communicate(timeout=TIMEOUT)
            finally:
                child.kill()
                child.wait()
        # The binary has exited, so its remaining output is already in the terminal.
        while chunk := read(master, 0.2):
            shown += chunk
        restored = termios.tcgetattr(slave) == before
        assert not read(slave, 0), "unread terminal input would reach the shell"
        return child.returncode, stdout, shown.decode().replace("\r\n", "\n"), restored
    finally:
        os.close(master)
        os.close(slave)


def type_key(keys):
    return run_on_terminal(("Type yes to sign: ", b"yes\n"), (KEY_PROMPT, keys))


class Harness(unittest.TestCase):
    def test_closes_terminal_after_prompt_timeout(self):
        master, slave = os.openpty()
        try:
            with patch("os.openpty", return_value=(master, slave)), patch(
                f"{__name__}.TIMEOUT", 0
            ):
                with self.assertRaisesRegex(AssertionError, "not shown"):
                    run_on_terminal(("a prompt that never appears", b""))
            for fd in (master, slave):
                with self.assertRaises(OSError):
                    os.fstat(fd)
        finally:
            for fd in (master, slave):
                try:
                    os.close(fd)
                except OSError:
                    pass


class ArgumentAndFileErrors(unittest.TestCase):
    def test_bad_arguments_exit_2_without_echoing_them(self):
        for args in [(), ("--params",), ("--params", PARAMS, "--key", KEY_1_WIF), (PARAMS,)]:
            with self.subTest(args=args):
                code, stdout, stderr = run_piped(*args)
                self.assertEqual((code, stdout), (2, b""))
                self.assertTrue(stderr.startswith("error: "), stderr)
                self.assertNotIn(KEY_1_WIF, stderr)

    def test_bad_params_files_fail_before_the_summary(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            (tmp / "not.json").write_text("not json")
            mainnet = json.loads(PARAMS.read_text()) | {"network": "bitcoin"}
            (tmp / "mainnet.json").write_text(json.dumps(mainnet))
            for params, message in [
                (
                    tmp / "missing.json",
                    f'failed to open params file "{tmp}/missing.json": No such file or directory',
                ),
                (tmp, "failed to read params: Is a directory"),
                (tmp / "not.json", "invalid params JSON: "),
                (tmp / "mainnet.json", 'unsupported network "bitcoin"'),
            ]:
                with self.subTest(params=params.name):
                    code, stdout, stderr = run_piped("--params", params)
                    self.assertEqual((code, stdout), (1, b""))
                    self.assertTrue(stderr.startswith(f"error: {message}"), stderr)

    def test_piped_stdin_cannot_confirm(self):
        self.assertEqual(
            run_piped("--params", PARAMS),
            (1, b"", f"{SUMMARY}error: terminal I/O failed: stdin is not a terminal\n"),
        )


class Terminal(unittest.TestCase):
    def test_signs_without_showing_the_key(self):
        # Enter as typed, and with a mistyped character erased by Backspace.
        for keys in [f"{KEY_1_WIF}\r", f"{KEY_1_WIF}x\x7f\r"]:
            with self.subTest(keys=keys):
                self.assertEqual(
                    type_key(keys.encode()), (0, f"{SIGNED_HEX}\n".encode(), KEY_ENTERED, True)
                )

    def test_rejects_bad_keys(self):
        invalid = "private key is not a valid WIF"
        for wif, error in [
            ("", invalid),
            ("not a key", invalid),
            (f"{KEY_1_WIF[:-1]}B", invalid),  # bad checksum
            (KEY_1_MAINNET_WIF, "private key is for mainnet; only testnet keys are supported"),
            (
                KEY_1_UNCOMPRESSED_WIF,
                "private key is uncompressed; P2WPKH requires a compressed key",
            ),
            (KEY_2_WIF, "private key does not control the previous output"),
        ]:
            with self.subTest(wif=wif):
                self.assertEqual(
                    type_key(f"{wif}\r".encode()),
                    (1, b"", f"{KEY_ENTERED}error: {error}\n", True),
                )

    def test_cancels_at_the_key_prompt(self):
        # Ctrl-C arrives as a byte there, so the terminal is restored before exiting.
        for keys in [b"cMah\x03", b"cMah\x04"]:
            with self.subTest(keys=keys):
                self.assertEqual(
                    type_key(keys), (1, b"", f"{KEY_ENTERED}error: signing cancelled\n", True)
                )

    def test_discards_unread_input_at_the_key_prompt(self):
        key_line = f"{KEY_1_WIF}\n".encode()
        for first, code, stdout, error in [
            (b"\n", 1, b"", "private key is not a valid WIF"),
            (b"\r", 1, b"", "private key is not a valid WIF"),
            (b"\x03", 1, b"", "signing cancelled"),
            (b"\x04", 1, b"", "signing cancelled"),
            (key_line, 0, f"{SIGNED_HEX}\n".encode(), ""),
        ]:
            with self.subTest(first=first):
                shown = KEY_ENTERED + (f"error: {error}\n" if error else "")
                # Paste in one write so the second line is already queued when the first ends.
                self.assertEqual(type_key(first + key_line), (code, stdout, shown, True))

    def test_cancels_at_the_confirmation(self):
        for keys, shown in [(b"no\n", "no\n"), (b"yes \n", "yes \n"), (b"\x04", "")]:
            with self.subTest(keys=keys):
                self.assertEqual(
                    run_on_terminal(("Type yes to sign: ", keys)),
                    (1, b"", f"{SUMMARY}{shown}error: signing cancelled\n", True),
                )

    def test_ctrl_c_at_the_confirmation_kills_it(self):
        self.assertEqual(
            run_on_terminal(("Type yes to sign: ", b"\x03")),
            (-signal.SIGINT, b"", f"{SUMMARY}^C", True),
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
