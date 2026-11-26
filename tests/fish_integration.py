"""Exercise the fish integration and built Rust binary in a real terminal."""

import os
from pathlib import Path
import shlex
import shutil
import sys
import tempfile
import unittest

from shell_pty import InteractiveShell


ROOT = Path(__file__).resolve().parent.parent
BINARY = Path(sys.argv.pop(1)).resolve() if len(sys.argv) > 1 else ROOT / "target/debug/t-helper"
START = f"source {shlex.quote(str(ROOT / 'shell/t.fish'))}"
UNAVAILABLE = "No previous command is available to diagnose."
PROMPT_SETUP = (
    "set -e fish_private_mode; set -e fish_history; "
    "function fish_greeting; end; function fish_prompt; printf 'fish-test> '; end; "
    # A separate event emits the status marker once. Fish can repaint the prompt itself.
    "function test_checkpoint --on-event fish_prompt; printf '\\n__t_test_%s__ \\n' $status; end"
)


class FishTest(unittest.TestCase):
    def setUp(self):
        fish = shutil.which("fish")
        if not fish:
            raise RuntimeError("fish is required to run the interactive tests")
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        bin_dir = Path(temp.name) / "bin"
        bin_dir.mkdir()
        (bin_dir / "t-helper").symlink_to(BINARY)
        self.shell = InteractiveShell(
            [fish, "--no-config", "--interactive", "--init-command", PROMPT_SETUP],
            {
                "PATH": os.pathsep.join([str(bin_dir), "/usr/bin", "/bin"]),
                "HOME": temp.name,
                "XDG_CONFIG_HOME": str(Path(temp.name) / "config"),
                "XDG_DATA_HOME": str(Path(temp.name) / "data"),
                "TERM": "dumb",
                "LC_ALL": "C",
            },
        )
        self.addCleanup(self.shell.close)
        self.shell.read_prompt()

    def run_line(self, line, status=0, contains=None):
        output, actual = self.shell.command(line)
        self.assertEqual(status, actual, output)
        if contains:
            self.assertIn(contains, output)
        return output

    def test_failure_success_and_repeat(self):
        self.run_line(START)
        self.run_line("t", contains=UNAVAILABLE)
        self.run_line("function git; echo 'fixture ran'; return 1; end")
        self.run_line("git statsu", status=1, contains="fixture ran")
        output = self.run_line("t", contains="Try: git status")
        self.assertNotIn("fixture ran", output)
        self.run_line("t", contains="The previous command succeeded.")
        self.run_line("false", status=1)
        self.run_line("true")
        self.run_line("t", contains="The previous command succeeded.")

    def test_uses_pipeline_status_and_ignores_status_carryover(self):
        self.run_line(START)
        self.run_line("true | false", status=1)
        self.run_line("t", contains="Exit status 1. Cannot tell why")
        self.run_line("false", status=1)
        self.run_line("# comment only", status=1)
        self.run_line("t", contains=UNAVAILABLE)

    def test_leading_space_opts_out_of_capture(self):
        self.run_line(START)
        self.run_line("false", status=1)
        self.run_line(" false private-marker", status=1)
        self.run_line("t", contains=UNAVAILABLE)

    def test_private_mode_and_an_empty_history_name_opt_out(self):
        self.run_line(START)
        self.run_line("set -g fish_private_mode 1; true")
        self.run_line("false", status=1)
        self.run_line("t", contains=UNAVAILABLE)
        # Even when private mode ends on this line, don't record the whole line.
        self.run_line("set -eg fish_private_mode; set -eg fish_history; false", status=1)
        self.run_line("t", contains=UNAVAILABLE)
        self.run_line("false", status=1)
        self.run_line("t", contains="Exit status 1. Cannot tell why")
        self.run_line("set -g fish_history ''; true")
        self.run_line("false", status=1)
        self.run_line("t", contains=UNAVAILABLE)

    def test_syntax_error_does_not_reuse_old_failure(self):
        self.run_line(START)
        self.run_line("false", status=1)
        # Fish leaves invalid input on the editing line. Ctrl-U replaces it with t.
        output = self.run_line("true )\n\x15t", contains=UNAVAILABLE)
        self.assertIn("true )", output)
        self.assertIn("parenthesis", output)

    def test_other_handlers_do_not_change_the_captured_status(self):
        self.run_line("function test_handler --on-event fish_postexec; false; end")
        self.run_line(START)
        self.run_line(START)
        self.run_line("gti status", status=127)
        self.run_line("t", contains="Exit status 127. `gti` looks like a typo for `git`.")

    def test_does_not_replace_an_existing_function(self):
        self.run_line("function t; echo 'existing function'; end")
        self.run_line(START, status=1, contains="t already exists")
        self.run_line("t", contains="existing function")


if __name__ == "__main__":
    if not BINARY.is_file():
        raise SystemExit(f"Build t-helper first: no binary at {BINARY}")
    unittest.main(verbosity=2)
