use std::{
    env,
    io::{stdin, BufRead, BufReader, Error, ErrorKind, Read},
    process,
};

use header::{validate, CommitMessage};
use prettytable::{format, row, Table};

use crate::header::default_commit_types;

mod header;

fn parse_stream<R: Read>(
    message_stream: BufReader<R>,
) -> Result<(String, String, String, String), Error> {
    let mut lines = message_stream.lines();
    let first_line = lines
        .next()
        .ok_or(Error::new(
            ErrorKind::InvalidData,
            "Failed to read first line",
        ))?
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    let parsed_header = header::parse(&first_line)?;
    let parsed = lines.fold("".to_string(), |mut acc, lines| {
        acc.push_str(lines.unwrap_or("".to_string()).as_str());
        acc.push_str("\n");
        acc
    });

    Ok((
        parsed_header.0,
        parsed_header.1,
        parsed_header.2,
        parsed.trim().to_string(),
    ))
}

pub fn parse<R: Read>(message: R) -> Result<(String, String, String, String), Error> {
    let message_stream = BufReader::new(message);
    let parsed = parse_stream(message_stream);
    parsed
}

fn parse_commit_types(text: String) -> Vec<CommitMessage> {
    let mut commit_messages: Vec<CommitMessage> = vec![];
    if text.is_empty() {
        return commit_messages;
    }
    for item in text.split(";").map(|s| s.to_string()) {
        let parts: Vec<String> = item.split("=").map(|s| s.to_string()).collect();
        let commit_type = parts[0].to_string();
        let required: Vec<String> = {
            let part = parts.get(1).unwrap_or(&"".to_string()).to_string();
            part.split(",")
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        commit_messages.push(CommitMessage {
            commit_type,
            required,
        });
    }
    commit_messages
}

fn parse_args() -> Result<(bool, bool, Vec<CommitMessage>), Error> {
    let mut dont_exit_on_errors = false;
    let mut allow_caps_type = false;
    let mut commit_types = default_commit_types();

    for (index, argument) in env::args().enumerate() {
        match argument.as_str() {
            "--dont-exit-on-errors" | "-e" => {
                dont_exit_on_errors = true;
            }
            "--allow-caps-types" | "-c" => {
                allow_caps_type = true;
            }
            "--types" | "-t" => match env::args().nth(index + 1) {
                Some(arg) => commit_types = parse_commit_types(arg),
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Missing argument for types",
                    ))
                }
            },
            _ => continue,
        }
    }

    Ok((dont_exit_on_errors, allow_caps_type, commit_types.to_vec()))
}

fn main() {
    let (dont_exit_on_errors, allow_caps_type, commit_types) = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            println!("Error!: {:#?}", err);
            process::exit(1);
        }
    };

    let syntax_tree = match parse(stdin()) {
        Ok(result) => result,
        Err(err) => {
            if !dont_exit_on_errors {
                println!("Error!: {:?}", err);
                process::exit(1);
            }
            (
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
        }
    };
    let validation = match validate(commit_types, allow_caps_type, &syntax_tree.0, &syntax_tree.1, &syntax_tree.2) {
        Ok(result) => result,
        Err(err) => {
            if !dont_exit_on_errors {
                println!("Error!: {:#?}", err);
                process::exit(1);
            }
            false
        }
    };
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_DEFAULT);
    table.set_titles(row!["Type", "Scope", "Description", "Body", "Valid"]);
    table.add_row(row![
        syntax_tree.0,
        syntax_tree.1,
        syntax_tree.2,
        syntax_tree.3,
        validation
    ]);
    table.printstd();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_valid_input() {
        let input = b"feat(module): Add a new feature.\nThis is the first line of the feature.\nAnd this is the last line.";
        let expected_output = (
            "feat".to_string(),
            "module".to_string(),
            "Add a new feature.".to_string(),
            "This is the first line of the feature.\nAnd this is the last line.".to_string(),
        );
        let result = parse(Cursor::new(input)).unwrap();
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_parse_invalid_input_with_fixup() {
        let input = b"fixup! fix: This is a fixup commit.\nThis is another line of the commit.\nAnd this is the last line.";
        let expected_output = ErrorKind::InvalidData;
        let result = parse(Cursor::new(input)).unwrap_err().kind();
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = b"This is not a valid input because it does not start with a keyword.";
        let expected_output = ErrorKind::InvalidData;
        let result = parse(Cursor::new(input)).unwrap_err().kind();
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_parse_input_without_colon() {
        let input = b"fixup! fix This is a fixup commit.\nThis is another line of the commit.\nAnd this is the last line.";
        let expected_output = ErrorKind::InvalidData;
        let result = parse(Cursor::new(input)).unwrap_err().kind();
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_parse_input_without_parenthesis() {
        let input = b"feature module: Add a new feature.\nThis is the first line of the feature.\nAnd this is the last line.";
        let expected_output = ErrorKind::InvalidData;
        let result = parse(Cursor::new(input)).unwrap_err().kind();
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_parse_commit_types() {
        // Test case 1: Check that the function can parse a commit type with no required fields
        let text = "fix=".to_string();
        let expected_output = vec![CommitMessage {
            commit_type: "fix".to_string(),
            required: vec![],
        }];
        assert_eq!(parse_commit_types(text), expected_output);

        // Test case 2: Check that the function can parse a commit type with required fields
        let text = "fix=field1,field2".to_string();
        let expected_output = vec![CommitMessage {
            commit_type: "fix".to_string(),
            required: vec!["field1".to_string(), "field2".to_string()],
        }];
        assert_eq!(parse_commit_types(text), expected_output);

        // Test case 3: Check that the function can parse multiple commit types
        let text = "fix=field1,field2;feature=field3,field4".to_string();
        let expected_output = vec![
            CommitMessage {
                commit_type: "fix".to_string(),
                required: vec!["field1".to_string(), "field2".to_string()],
            },
            CommitMessage {
                commit_type: "feature".to_string(),
                required: vec!["field3".to_string(), "field4".to_string()],
            },
        ];
        assert_eq!(parse_commit_types(text), expected_output);

        // Test case 4: Check that the function can handle empty input
        let text = "".to_string();
        let expected_output = vec![];
        assert_eq!(parse_commit_types(text), expected_output);
    }
}
