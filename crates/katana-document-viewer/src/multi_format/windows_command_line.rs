pub(crate) struct WindowsCommandLine;

impl WindowsCommandLine {
    pub(crate) fn from_arguments<I, S>(arguments: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        arguments
            .into_iter()
            .map(|argument| quote_argument(argument.as_ref()))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn quote_argument(argument: &str) -> String {
    let mut quoted = String::from("\"");
    let mut backslashes = 0_usize;
    for character in argument.chars() {
        match character {
            '\\' => backslashes = backslashes.saturating_add(1),
            '"' => {
                push_backslashes(&mut quoted, backslashes.saturating_mul(2).saturating_add(1));
                quoted.push('"');
                backslashes = 0;
            }
            _ => {
                push_backslashes(&mut quoted, backslashes);
                quoted.push(character);
                backslashes = 0;
            }
        }
    }
    push_backslashes(&mut quoted, backslashes.saturating_mul(2));
    quoted.push('"');
    quoted
}

fn push_backslashes(output: &mut String, count: usize) {
    for _ in 0..count {
        output.push('\\');
    }
}

#[cfg(test)]
mod tests {
    use super::WindowsCommandLine;

    #[test]
    fn quotes_spaces_quotes_and_trailing_backslashes() {
        let command = WindowsCommandLine::from_arguments([
            r"C:\Program Files\KDV\worker.exe",
            r#"C:\tmp\quoted"name\"#,
            "",
        ]);
        assert_eq!(
            command,
            r#""C:\Program Files\KDV\worker.exe" "C:\tmp\quoted\"name\\" """#
        );
    }
}
