use std::io::{Error, ErrorKind};

#[derive(PartialEq, Debug)]
pub(crate) enum State {
    Type,
    Scope,
    Description,
    Body,
}

pub fn parse(line: &str) -> Result<(String, String, String), Error> {
    let mut _type = String::new();
    let mut _scope = String::new();
    let mut _description = String::new();
    let mut state = State::Type;
    let mut paren_count = 0;
    let mut valid_scope = false;
    for c in line.chars() {
        match state {
            State::Type => {
                if c.is_alphanumeric() || c == '_' {
                    _type.push(c);
                } else if c == '(' {
                    state = State::Scope;
                    paren_count += 1;
                } else if c == ':' {
                    state = State::Description;
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Incorrect commit message, expected format 'TYPE([SCOPE]): MESSAGE\n[BODY]'! Failed to read the type from the header",
                    ));
                }
            }
            State::Scope => {
                if c.is_alphanumeric()
                    || c == '_'
                    || c == ','
                    || c == '$'
                    || c == '.'
                    || c == '/'
                    || c == '-'
                {
                    _scope.push(c);
                } else if c == ')' {
                    valid_scope = true;
                } else if c == ':' {
                    paren_count -= 1;
                    if paren_count == 0 {
                        state = State::Description;
                    }
                    if !valid_scope {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Incorrect commit message, expected format 'TYPE([SCOPE]): MESSAGE\n[BODY]'!! Failed to retrieve the scope from the header",
                        ));
                    }
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Incorrect commit message, expected format 'TYPE([SCOPE]): MESSAGE\n[BODY]'!! Failed to retrieve the scope from the header",
                    ));
                }
            }
            State::Description => {
                if c != '\n' {
                    _description.push(c);
                } else {
                    state = State::Body;
                    break;
                }
            }
            _ => {}
        }
    }
    if state != State::Body && state != State::Description {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Incorrect commit message, expected format 'TYPE([SCOPE]): MESSAGE\n[BODY]'!! Failed to read the body, ended up with the state {:?} instead.",
                state
            ),
        ));
    }
    Ok((
        _type.trim().to_string(),
        _scope.trim().to_string(),
        _description.trim().to_string(),
    ))
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommitMessage {
    pub(crate) commit_type: String,
    pub(crate) required: Vec<String>,
}

pub(crate) fn default_commit_types() -> Vec<CommitMessage> {
    vec![
        CommitMessage {
            commit_type: "feat".to_string(),
            required: vec!["scope".to_string(), "description".to_string()],
        },
        CommitMessage {
            commit_type: "fix".to_string(),
            required: vec!["scope".to_string(), "description".to_string()],
        },
        CommitMessage {
            commit_type: "build".to_string(),
            required: vec!["description".to_string()],
        },
        CommitMessage {
            commit_type: "chore".to_string(),
            required: vec!["description".to_string()],
        },
        CommitMessage {
            commit_type: "ci".to_string(),
            required: vec!["description".to_string()],
        },
        CommitMessage {
            commit_type: "docs".to_string(),
            required: vec!["description".to_string()],
        },
        CommitMessage {
            commit_type: "perf".to_string(),
            required: vec!["description".to_string()],
        },
        CommitMessage {
            commit_type: "refactor".to_string(),
            required: vec!["description".to_string()],
        },
        CommitMessage {
            commit_type: "revert".to_string(),
            required: vec!["description".to_string()],
        },
        CommitMessage {
            commit_type: "style".to_string(),
            required: vec!["description".to_string()],
        },
        CommitMessage {
            commit_type: "test".to_string(),
            required: vec!["description".to_string()],
        },
    ]
}

pub fn validate(
    spec: Vec<CommitMessage>,
    commit_type: &str,
    scope: &str,
    description: &str,
) -> Result<bool, Error> {
    let commit_type = spec.iter().find(|x| x.commit_type == commit_type);
    match commit_type {
        Some(_type) => {
            if _type.required.contains(&"scope".to_string()) && scope.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Commit type requires a scope, but none given",
                ));
            }
            if _type.required.contains(&"description".to_string()) && description.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Commit type requires a description, but none given",
                ));
            }
            return Ok(true);
        }
        None => Err(Error::new(
            ErrorKind::InvalidData,
            "Commit type not allowed",
        )),
    }
}

#[test]
fn test_header_success() {
    let test_cases = vec![
        (
            "name:".to_string(),
            ("name".to_string(), "".to_string(), "".to_string()),
        ),
        (
            "name(args): ".to_string(),
            ("name".to_string(), "args".to_string(), "".to_string()),
        ),
        (
            "name: value".to_string(),
            ("name".to_string(), "".to_string(), "value".to_string()),
        ),
        (
            "name(args): value".to_string(),
            ("name".to_string(), "args".to_string(), "value".to_string()),
        ),
        (
            "name(args): value: another_value".to_string(),
            (
                "name".to_string(),
                "args".to_string(),
                "value: another_value".to_string(),
            ),
        ),
        (
            "name(arg1,arg2): value".to_string(),
            (
                "name".to_string(),
                "arg1,arg2".to_string(),
                "value".to_string(),
            ),
        ),
        (
            "name(arg_1,arg-2,arg$3): value".to_string(),
            (
                "name".to_string(),
                "arg_1,arg-2,arg$3".to_string(),
                "value".to_string(),
            ),
        ),
    ];
    for (input, expected) in test_cases {
        match parse(&input) {
            Ok(header) => assert_eq!(header, expected, "Unexpected failure for value {}", input),
            Err(e) => assert!(
                false,
                "Should not have failed for '{}', but did with '{}'",
                input, e
            ),
        }
    }
}

#[test]
fn test_header_failure() {
    let test_cases = vec![
        "",
        "name",
        "name value",
        "name(args) value",
        "name(args: value",
        "name(arg.1/2*3): value",
    ];
    for input in test_cases {
        match parse(&input) {
            Ok(_) => assert!(false, "Should not have failed for '{}', but didn't", input),
            Err(e) => assert_eq!(ErrorKind::InvalidData, e.kind()),
        }
    }
}

#[test]
fn test_validate_success() {
    let test_cases = vec![
        ("feat", "scope", "description"),
        ("fix", "scope", "description"),
        ("build", "", "description"),
        ("build", "scope", "description"),
        ("chore", "", "description"),
        ("ci", "", "description"),
        ("docs", "", "description"),
        ("perf", "", "description"),
        ("refactor", "", "description"),
        ("revert", "", "description"),
        ("style", "", "description"),
        ("test", "", "description"),
    ];
    for (commit_type, scope, description) in test_cases {
        let result = validate(default_commit_types(), commit_type, scope, description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}

#[test]
fn test_validate_failure() {
    let test_cases = vec![
        ("not_allowed", "scope", "description"),
        ("feat", "", "description"),
        ("fix", "", "description"),
        ("build", "scope", ""),
    ];
    for (commit_type, scope, description) in test_cases {
        let result = validate(default_commit_types(), commit_type, scope, description);
        assert!(result.is_err());
    }
}
