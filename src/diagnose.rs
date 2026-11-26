mod rules;
mod words;

#[derive(Debug, PartialEq, Eq)]
pub enum Advice {
    Success,
    // These fields are static so a rule cannot print raw user input.
    Typo {
        wrong: &'static str,
        right: &'static str,
        example: Option<&'static str>,
    },
    Hint {
        message: &'static str,
        check: Option<&'static str>,
    },
    Unknown,
}

pub fn diagnose(command: &str, status: u8) -> Advice {
    if status == 0 {
        return Advice::Success;
    }
    let Some(words) = words::literal_words(command) else {
        // A compound command can report the status of just one part.
        return Advice::Unknown;
    };
    let words: Vec<_> = words.iter().map(String::as_str).collect();
    let [tool, args @ ..] = words.as_slice() else {
        return Advice::Unknown;
    };

    match status {
        127 => rules::command_not_found(tool),
        126 => Advice::Hint {
            message: "If your shell could not execute the command, check its permissions and file type.",
            check: None,
        },
        130 => Advice::Hint {
            message: "This status often means the command was interrupted, for example by Ctrl-C.",
            check: None,
        },
        1 => rules::exact(tool, args).unwrap_or_else(|| rules::context(tool, args)),
        2..=125 | 128 => rules::context(tool, args),
        _ => Advice::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{Advice, diagnose};

    #[test]
    fn fixes_known_typos_only_for_plausible_statuses() {
        assert_eq!(
            diagnose("kubctl get pods", 127),
            Advice::Typo {
                wrong: "kubctl",
                right: "kubectl",
                example: None
            }
        );
        assert_eq!(
            diagnose("git statsu", 1),
            Advice::Typo {
                wrong: "statsu",
                right: "status",
                example: Some("git status")
            }
        );
        assert_eq!(diagnose("kubctl get pods", 1), Advice::Unknown);
        assert_eq!(diagnose("git statsu", 0), Advice::Success);
        assert!(!matches!(diagnose("git statsu", 127), Advice::Typo { .. }));
    }

    #[test]
    fn missing_legacy_compose_command_suggests_checking_for_plugin() {
        assert!(matches!(
            diagnose("docker-compose up", 127),
            Advice::Hint {
                check: Some("docker compose version"),
                ..
            }
        ));
    }

    #[test]
    fn tool_rules_preserve_arguments_by_suggesting_a_word_edit() {
        let cases = [
            ("git checout private-branch", "checout", "checkout"),
            ("brew instal jq", "instal", "install"),
            ("docker comopse ps", "comopse", "compose"),
            ("docker compose strat example", "strat", "start"),
            ("kubectl descirbe pod test", "descirbe", "describe"),
        ];
        for (command, wrong, right) in cases {
            assert_eq!(
                diagnose(command, 1),
                Advice::Typo {
                    wrong,
                    right,
                    example: None
                }
            );
        }
    }

    #[test]
    fn ambiguous_failures_offer_checks_without_asserting_a_cause() {
        let cases = [
            ("git push", 128, Some("git status")),
            ("git push", 1, Some("git status")),
            ("git status", 1, Some("git rev-parse --show-toplevel")),
            ("git rev-parse --show-toplevel", 128, None),
            ("brew info jq", 1, Some("brew config")),
            ("docker ps", 1, Some("docker context show")),
            (
                "docker compose up",
                125,
                Some("docker compose config --quiet"),
            ),
            (
                "kubectl get pods",
                1,
                Some("kubectl config current-context"),
            ),
        ];
        for (command, status, check) in cases {
            let advice = diagnose(command, status);
            if check.is_none() {
                assert_eq!(advice, Advice::Unknown);
            } else {
                assert!(matches!(advice, Advice::Hint { check: actual, .. } if actual == check));
            }
        }
    }

    #[test]
    fn pipelines_expansions_and_wrappers_do_not_trigger_typo_rules() {
        for command in [
            "kubctl get pods | cat",
            "false; git statsu",
            "git statsu && true",
            "git statsu > file",
            "git statsu $(some-command)",
            "git statsu\nfalse",
            "sudo git statsu",
            "ENV=x git statsu",
        ] {
            assert_eq!(diagnose(command, 1), Advice::Unknown, "{command}");
        }
    }

    #[test]
    fn bsd_sed_rule_is_specific_to_the_macos_system_binary() {
        let advice = diagnose("/usr/bin/sed -i 's/old/new/' 'my file'", 1);
        assert_eq!(
            matches!(advice, Advice::Hint { .. }),
            cfg!(target_os = "macos")
        );
        assert_eq!(diagnose("sed -i 's/old/new/' file", 1), Advice::Unknown);
        assert_eq!(
            diagnose("/usr/bin/sed -i '' 's/old/new/' file", 1),
            Advice::Unknown
        );
    }
}
