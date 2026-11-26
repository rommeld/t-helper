/// Recognize literal words with simple quotes, without interpreting shell syntax.
/// If syntax could affect which command ran, let the caller use a generic hint.
pub(super) fn literal_words(input: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut quote = None;

    for ch in input.chars() {
        if ch.is_control() && ch != '\t' {
            return None;
        }
        match (quote, ch) {
            (Some(delimiter), ch) if ch == delimiter => quote = None,
            (Some('"'), '$' | '`' | '\\' | '!') => return None,
            (Some(_), ch) => word.push(ch),
            (None, ' ' | '\t') => {
                if in_word {
                    words.push(std::mem::take(&mut word));
                    in_word = false;
                }
            }
            (None, '\'' | '"') => {
                in_word = true;
                quote = Some(ch);
            }
            (
                None,
                ';' | '|' | '&' | '<' | '>' | '(' | ')' | '{' | '}' | '$' | '`' | '\\' | '*' | '?'
                | '[' | ']' | '~' | '!' | '#' | '^',
            ) => return None,
            (None, ch) => {
                in_word = true;
                word.push(ch);
            }
        }
    }

    if quote.is_some() {
        return None;
    }
    if in_word {
        words.push(word);
    }
    if words
        .first()
        .is_some_and(|word| word.is_empty() || word.contains('='))
    {
        return None;
    }
    Some(words)
}

#[cfg(test)]
mod tests {
    use super::literal_words;

    #[test]
    fn recognizes_quoted_literal_words() {
        let words = literal_words("\tgit 'sta'tus \"a file\" '' '$(not-expanded);' --flag=value");
        assert_eq!(
            words.unwrap(),
            [
                "git",
                "status",
                "a file",
                "",
                "$(not-expanded);",
                "--flag=value"
            ]
        );
    }

    #[test]
    fn declines_shell_syntax_and_unclosed_quotes() {
        for input in [
            "git \"$ARG\"",
            "git `tool`",
            "git 'unterminated",
            "git \\arg",
            "git #comment",
            "git *.txt",
            "git \u{1b}[2J",
            "X=1 git status",
            "=git status",
        ] {
            assert!(literal_words(input).is_none(), "{input}");
        }
    }
}
