# t-helper

Type `t` at the next zsh prompt after a failed command. `t` suggests a fix or a check based on the command and its exit status. It never runs suggestions for you.

```text
% git statsu
git: 'statsu' is not a git command. See 'git --help'.
% t
Exit status 1. `statsu` looks like a typo for `status`.
Try: git status
```

## Install and use `t`

You need zsh and Rust 1.85 or later. The first release targets macOS. CI also tests the zsh integration on Linux.

1. Clone this repo and install the binary:

	```sh
	git clone https://github.com/rommeld/t-helper.git
	cd t-helper
	cargo install --locked --path .
	```

2. Make sure `t-helper` is on `PATH`. Cargo uses `$HOME/.cargo/bin` by default.
3. Add a line like this to `~/.zshrc` after your prompt setup. Replace the example path with the path to your checkout:

	```zsh
	source "$HOME/src/t-helper/shell/t.zsh"
	```

4. Open a new zsh session. Run `git statsu` and then `t` at the next prompt.

If you already have a command or alias named `t`, rename or remove it before you load the integration.

## Know what `t` can diagnose

`t` recognizes common shell mistakes, selected macOS command patterns, and known typos in Git, Homebrew, Docker, Docker Compose, and kubectl. When a tool's exit status is ambiguous, `t` can suggest a check such as `docker context show`. For pipelines and compound commands, it reports the overall status without guessing which part failed.

`t` keeps the latest command context in the current shell. It checks that the exact history event is still in zsh's in-memory history before sending the command to the local rule engine. It does not search for older commands, record output, write logs, or use the network.

`t` skips commands zsh drops from its in-memory history. That includes leading-space commands and alias expansions with `HIST_IGNORE_SPACE` enabled, and commands rejected by a `zshaddhistory` hook. A hook that only prevents writing a command to the history file leaves it available in memory. If a prompt hook fails or `t` cannot verify the history event, it does not reuse an older failure.

The diagnosis uses the command as typed. `t` does not resolve aliases, wrapper commands such as `sudo`, or substitutions. A syntax error can return to the prompt before zsh tells `t` what command was entered. In that case, `t` has no command to diagnose.

## Run the tests

You need zsh and Python 3.8 or later for the interactive tests. You do not need Homebrew, Docker, or kubectl.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked
python3 tests/zsh_integration.py target/debug/t-helper
```
