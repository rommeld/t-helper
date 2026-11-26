# t-helper

Type `t` at the next fish or zsh prompt after a failed command. `t` suggests a fix or a check based on the command and its exit status. It never runs suggestions for you.

```text
> git statsu
git: 'statsu' is not a git command. See 'git --help'.
> t
Exit status 1. `statsu` looks like a typo for `status`.
Try: git status
```

## Install and use `t`

You need fish 3.7 or later, or zsh. To build the binary, you need Rust 1.85 or later. The integrations work on macOS and Linux.

1. Clone this repo and install the binary:

	```sh
	git clone https://github.com/rommeld/t-helper.git
	cd t-helper
	cargo install --locked --path .
	```

2. Make sure `t-helper` is on `PATH`. Cargo uses `$HOME/.cargo/bin` by default.
3. Add the `source` line for your shell. Replace `$HOME/src/t-helper` with the path to your checkout.

	For fish, add this line to `~/.config/fish/config.fish`:

	```fish
	source "$HOME/src/t-helper/shell/t.fish"
	```

	For zsh, add this line to `~/.zshrc` after your prompt setup:

	```zsh
	source "$HOME/src/t-helper/shell/t.zsh"
	```

4. Open a new shell session. Run `git statsu` and then `t` at the next prompt.

If you already have a command or alias named `t`, rename or remove it before you load the integration.

## Know what `t` can diagnose

`t` recognizes common shell mistakes, selected macOS command patterns, and known typos in Git, Homebrew, Docker, Docker Compose, and kubectl. When a tool's exit status is ambiguous, `t` can suggest a check such as `docker context show`. For pipelines and compound commands, it reports the overall status without guessing which part failed.

`t` keeps the latest command context in the current shell. It does not search for older commands, record output, write logs, or use the network.

- In fish, `t` reads command events rather than shell history. It skips commands in private mode, commands that start with a space, and commands run with `fish_history` set to an empty string.
- In zsh, `t` verifies the exact event in in-memory history. It skips commands dropped by `HIST_IGNORE_SPACE` or a `zshaddhistory` hook. A hook that only prevents writing to the history file leaves the command available in memory. If the prompt capture fails, `t` does not reuse an older failure.

The diagnosis uses the command as typed. `t` does not resolve aliases, wrapper commands such as `sudo`, or substitutions. A syntax error can return to the prompt before zsh tells `t` what command was entered. In that case, `t` has no command to diagnose.

## Run the tests

You need fish, zsh, and Python 3.8 or later for the interactive tests. You do not need Homebrew, Docker, or kubectl.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked
python3 tests/fish_integration.py target/debug/t-helper
python3 tests/zsh_integration.py target/debug/t-helper
```
