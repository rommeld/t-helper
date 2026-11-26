use super::Advice;

pub(super) fn command_not_found(tool: &str) -> Advice {
    let (wrong, right) = match tool {
        "gti" => ("gti", "git"),
        "berw" => ("berw", "brew"),
        "doker" => ("doker", "docker"),
        "kubctl" => ("kubctl", "kubectl"),
        "pyhton3" => ("pyhton3", "python3"),
        "docker-compose" => {
            return Advice::Hint {
                message: "If `docker-compose` is missing, check whether Docker provides Compose as `docker compose` instead.",
                check: Some("docker compose version"),
            };
        }
        _ => {
            return Advice::Hint {
                message: "If your shell said 'command not found', check the spelling and whether the executable is on PATH.",
                check: None,
            };
        }
    };
    Advice::Typo {
        wrong,
        right,
        example: None,
    }
}

pub(super) fn exact(tool: &str, args: &[&str]) -> Option<Advice> {
    let (wrong, right, example) = match (tool, args) {
        ("git" | "/usr/bin/git", ["statsu", rest @ ..]) => {
            ("statsu", "status", rest.is_empty().then_some("git status"))
        }
        ("git" | "/usr/bin/git", ["checout", ..]) => ("checout", "checkout", None),
        ("git" | "/usr/bin/git", ["comit", ..]) => ("comit", "commit", None),
        ("brew", ["instal", ..]) => ("instal", "install", None),
        ("brew", ["udpate", ..]) => ("udpate", "update", None),
        ("docker", ["comopse", ..]) => ("comopse", "compose", None),
        ("docker", ["compose", "strat", ..]) => ("strat", "start", None),
        ("kubectl", ["descirbe", ..]) => ("descirbe", "describe", None),
        ("kubectl", ["appyl", ..]) => ("appyl", "apply", None),
        ("/usr/bin/sed", ["-i", script, file])
            if cfg!(target_os = "macos") && script.starts_with("s/") && !file.starts_with('-') =>
        {
            return Some(Advice::Hint {
                message: "On macOS, `/usr/bin/sed -i` takes a backup suffix. Use `-i.bak` to keep a backup, or omit `-i` to preview the edit.",
                check: None,
            });
        }
        _ => return None,
    };
    Some(Advice::Typo {
        wrong,
        right,
        example,
    })
}

pub(super) fn context(tool: &str, args: &[&str]) -> Advice {
    let (message, check) = match (tool, args) {
        ("git" | "/usr/bin/git", ["status", ..]) => (
            "The status does not identify the Git error. Check whether you are in a worktree.",
            "git rev-parse --show-toplevel",
        ),
        ("git" | "/usr/bin/git", ["rev-parse", "--show-toplevel", ..]) => return Advice::Unknown,
        ("git" | "/usr/bin/git", _) => (
            "The status does not identify the Git error. Check the repository and worktree state.",
            "git status",
        ),
        ("brew", ["config", ..]) => return Advice::Unknown,
        ("brew", _) => (
            "The status does not identify the Homebrew error. Check the Homebrew configuration.",
            "brew config",
        ),
        ("docker", ["compose", "config", ..] | ["context", "show", ..]) => return Advice::Unknown,
        ("docker", ["compose", ..]) => (
            "The status does not identify the Compose error. Check whether the configuration is valid.",
            "docker compose config --quiet",
        ),
        ("docker", _) => (
            "The status does not identify the Docker error. Check which Docker context is active.",
            "docker context show",
        ),
        ("kubectl", ["config", "current-context", ..]) => return Advice::Unknown,
        ("kubectl", _) => (
            "The status does not identify the Kubernetes error. Check which Kubernetes context is active.",
            "kubectl config current-context",
        ),
        _ => return Advice::Unknown,
    };
    Advice::Hint {
        message,
        check: Some(check),
    }
}
