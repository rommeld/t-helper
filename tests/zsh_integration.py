"""Exercise the zsh hook and built Rust binary through a real terminal."""

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


class ShellTest(unittest.TestCase):
    def setUp(self):
        zsh = shutil.which("zsh")
        if not zsh:
            raise RuntimeError("zsh is required to run the interactive tests")
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        bin_dir = Path(temp.name) / "bin"
        bin_dir.mkdir()
        (bin_dir / "t-helper").symlink_to(BINARY)
        self.shell = InteractiveShell(
            [zsh, "-f", "-i"],
            {
                "PATH": os.pathsep.join([str(bin_dir), "/usr/bin", "/bin"]),
                "HOME": temp.name,
                "ZDOTDIR": temp.name,
                "TERM": "dumb",
                "PS1": "__t_test_%?__ ",
                "LC_ALL": "C",
            },
        )
        self.addCleanup(self.shell.close)
        self.shell.read_prompt()
        self.start = f"source {shlex.quote(str(ROOT / 'shell/t.zsh'))}"

    def run_line(self, line, status=0, contains=None):
        output, actual = self.shell.command(line)
        if status is not None:
            self.assertEqual(status, actual, output)
        if contains:
            self.assertIn(contains, output)
        return output

    def test_failure_success_and_repeat(self):
        self.run_line(self.start)
        self.run_line("t", contains="No previous command")
        self.run_line("function git { print 'fixture ran'; return 1; }")
        self.run_line("git statsu", status=1, contains="fixture ran")
        output = self.run_line("t", contains="Try: git status")
        self.assertNotIn("fixture ran", output)
        self.run_line("t", contains="The previous command succeeded.")
        self.run_line("false", status=1)
        self.run_line("true")
        self.run_line("t", contains="The previous command succeeded.")

    def test_private_command_invalidates_older_failure(self):
        self.run_line("setopt HIST_IGNORE_SPACE")
        self.run_line(self.start)
        self.run_line("gti status", status=127)
        self.run_line(" kubctl get pods", status=127)
        output = self.run_line("t", contains="No previous command")
        self.assertNotIn("kubctl", output)
        self.assertNotIn("gti", output)

    def test_alias_that_expands_to_a_private_line_is_skipped(self):
        self.run_line("setopt HIST_IGNORE_SPACE")
        self.run_line("alias hidden_failure=' false'")
        self.run_line(self.start)
        self.run_line("gti status", status=127)
        self.run_line("hidden_failure", status=1)
        self.run_line("t", contains="No previous command")

    def test_history_hook_opt_out_is_skipped(self):
        self.run_line("zshaddhistory() { [[ $1 != 'false private-marker'* ]]; }")
        self.run_line(self.start)
        self.run_line("gti status", status=127)
        self.run_line("false private-marker", status=1)
        self.run_line("t", contains="No previous command")
        self.run_line("false", status=1)
        self.run_line("t", contains="Exit status 1. Cannot tell why")

    def test_history_check_handles_literal_metacharacters_and_duplicates(self):
        self.run_line("setopt HIST_IGNORE_DUPS")
        self.run_line(self.start)
        self.run_line("function git { return 1; }")
        for _ in range(2):
            self.run_line("git statsu '[file]*'", status=1)
        self.run_line("t", contains="Replace `statsu` with `status`")

    def test_shared_history_does_not_replace_the_local_capture(self):
        peer = ShellTest()
        self.addCleanup(peer.doCleanups)
        peer.setUp()
        with tempfile.TemporaryDirectory() as temp:
            history = shlex.quote(str(Path(temp) / "shared-history"))
            setup = f"HISTFILE={history}; SAVEHIST=100; setopt SHARE_HISTORY"
            self.run_line(setup)
            peer.run_line(setup)
            self.run_line(self.start)
            self.run_line("gti status", status=127)
            peer.run_line("true")
            self.run_line("t", contains="Exit status 127. `gti` looks like a typo for `git`.")

            self.run_line("zshaddhistory() { [[ $1 != 'false private-marker'* ]]; }")
            self.run_line("false private-marker", status=1)
            peer.run_line("false private-marker", status=1)
            self.run_line("t", contains="No previous command")

    def test_parse_error_never_reuses_previous_failure(self):
        self.run_line(self.start)
        self.run_line("gti status", status=127)
        self.run_line("true )", status=None, contains="parse error")
        self.run_line("t", contains="No previous command")

    def test_pipeline_reports_only_overall_status(self):
        self.run_line(self.start)
        self.run_line("gti | false", status=1)
        output = self.run_line("t", contains="Exit status 1. Cannot tell why")
        self.assertNotIn("git", output)

    def test_coexists_with_prompt_hooks_and_can_be_sourced_twice(self):
        self.run_line("precmd() { :; }; autoload -Uz add-zsh-hook")
        self.run_line("test_prompt_hook() { :; }; add-zsh-hook precmd test_prompt_hook")
        self.run_line(self.start)
        self.run_line(self.start)
        self.run_line("gti status", status=127)
        self.run_line("t", contains="Exit status 127. `gti` looks like a typo for `git`.")

    def test_prompt_hook_error_cannot_reuse_older_failure(self):
        self.run_line("autoload -Uz add-zsh-hook")
        self.run_line("prompt_error() { (( $? != 1 )) || ${missing:?prompt error}; }")
        self.run_line("add-zsh-hook precmd prompt_error")
        self.run_line(self.start)
        self.run_line("gti status", status=127)
        self.run_line("false", status=1)
        self.run_line("t", contains="No previous command")

    def test_does_not_replace_an_existing_alias(self):
        self.run_line("alias t='print -- existing-alias'")
        self.run_line(self.start, status=1, contains="t already exists")
        self.run_line("t", contains="existing-alias")

    def test_uses_the_binary_only_when_needed(self):
        self.run_line(self.start)
        self.run_line("PATH=/usr/bin:/bin; true")
        self.run_line("t", contains="The previous command succeeded.")
        self.run_line("false", status=1)
        self.run_line("t", status=1, contains="binary not found on PATH")


if __name__ == "__main__":
    if not BINARY.is_file():
        raise SystemExit(f"Build t-helper first: no binary at {BINARY}")
    unittest.main(verbosity=2)
