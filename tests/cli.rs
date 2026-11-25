use std::io::Write;
use std::process::{Command, Output, Stdio};

fn diagnose(input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_t-helper"))
        .arg("diagnose")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn prints_a_hint_without_echoing_arguments() {
    let output = diagnose(b"1\x00git statsu --token='PRIVATE-123'\x00");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Replace `statsu` with `status`"));
    assert!(!stdout.contains("PRIVATE-123"));
    assert!(!stdout.contains("Try: git status"));
}

#[test]
fn uses_the_same_safe_fallback_for_command_injection_syntax() {
    let output = diagnose(b"1\x00git statsu $(printf PRIVATE-123); false\x00");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Exit status 1. Cannot tell why from the command and status alone. Check the error above.\n",
    );
}

#[test]
fn rejects_invalid_requests_without_echoing_them() {
    let cases: &[&[u8]] = &[
        b"",
        b"1\x00PRIVATE-123",
        b"256\x00PRIVATE-123\x00",
        b"+1\x00PRIVATE-123\x00",
        b"1\x00\x00",
        b"1\x00PRIVATE-123\x00extra",
        b"1\x00\xffPRIVATE-123\x00",
    ];
    for input in cases {
        let output = diagnose(input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            !String::from_utf8(output.stderr)
                .unwrap()
                .contains("PRIVATE-123")
        );
    }
}

#[test]
fn rejects_oversize_requests() {
    let output = diagnose(&vec![b'x'; 128 * 1024 + 1]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("too long")
    );
}
