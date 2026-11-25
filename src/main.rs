mod diagnose;

use std::io::{self, IsTerminal, Read};
use std::process::ExitCode;

use diagnose::{Advice, diagnose};

const MAX_REQUEST_BYTES: usize = 128 * 1024;
const HELP: &str = "t-helper: local hints for failed zsh commands.\n\
    Source shell/t.zsh in your interactive zsh, then type t after a failure.";

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let action = args.next();
    if args.next().is_some() {
        return usage_error();
    }

    match action.as_deref().and_then(|arg| arg.to_str()) {
        None if action.is_none() => println!("{HELP}"),
        Some("-h" | "--help") => println!("{HELP}"),
        Some("--version") => println!("t-helper {}", env!("CARGO_PKG_VERSION")),
        Some("diagnose") => {
            let stdin = io::stdin();
            if stdin.is_terminal() {
                return usage_error();
            }
            let mut input = Vec::new();
            if let Err(error) = stdin
                .lock()
                .take((MAX_REQUEST_BYTES + 1) as u64)
                .read_to_end(&mut input)
            {
                eprintln!("t-helper: could not read the diagnosis request: {error}");
                return ExitCode::FAILURE;
            }
            match decode(&input) {
                Ok((status, command)) => show(status, diagnose(command, status)),
                Err(message) => {
                    eprintln!("t-helper: {message}");
                    return ExitCode::from(2);
                }
            }
        }
        _ => return usage_error(),
    }
    ExitCode::SUCCESS
}

fn usage_error() -> ExitCode {
    eprintln!("{HELP}");
    ExitCode::from(2)
}

// Private wire format: decimal status, NUL, original command text, NUL, EOF.
fn decode(input: &[u8]) -> Result<(u8, &str), &'static str> {
    if input.len() > MAX_REQUEST_BYTES {
        return Err("the previous command is too long to diagnose");
    }
    let mut fields = input.split(|&byte| byte == 0);
    let (Some(status), Some(command), Some([]), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err("invalid diagnosis request");
    };
    let status: u8 = std::str::from_utf8(status)
        .ok()
        .filter(|value| value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse().ok())
        .ok_or("invalid exit status")?;
    let command = std::str::from_utf8(command).map_err(|_| "the command is not valid UTF-8")?;
    if command.is_empty() {
        return Err("missing command");
    }
    Ok((status, command))
}

fn show(status: u8, advice: Advice) {
    match advice {
        Advice::Success => println!("The previous command succeeded."),
        Advice::Typo {
            wrong,
            right,
            example,
        } => {
            println!("Exit status {status}. `{wrong}` looks like a typo for `{right}`.");
            if let Some(example) = example {
                println!("Try: {example}");
            } else {
                println!("Replace `{wrong}` with `{right}` in the command.");
            }
        }
        Advice::Hint { message, check } => {
            println!("Exit status {status}. {message}");
            if let Some(check) = check {
                println!("Check: {check}");
            }
        }
        Advice::Unknown => println!(
            "Exit status {status}. Cannot tell why from the command and status alone. Check the error above."
        ),
    }
}
