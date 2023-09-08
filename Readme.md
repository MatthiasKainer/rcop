# RCOP - Rust preCOmmit COP

This package provides a command line utility to validate commit messages.

## Installation

To use this package, you'll need to have Rust and Cargo installed on your system. Once you have them, you can install this package by running the following command:

```rs
cargo install rcop
```

You can also download the latest executable and run them directly via

```bash
RELEASE_VERSION=0.1.2
RELEASE_OS=x86_64-unknown-linux-musl # or x86_64-apple-darwin
curl -L -o rcop https://github.com/MatthiasKainer/rcop/releases/download/$RELEASE_VERSION/rcop-$RELEASE_VERSION-$RELEASE_OS
chmod +x rcop
echo "chore: hello world" | ./rcop
```

## Usage

To use the command line utility, run the following command in the root of your git repository:

```sh
echo "docs: This is a first test" | rcop
```

This will read the commit message from the standard input, and then validate it based on the default commit types: fix, feat, docs, style, refactor, perf, test, and chore.

Add the location for `rcop` to your `$PATH` and the `commit-msg` from this repository into the `.git/hooks/` to verify the message.

## Options
Here are the command line options you can use with rcop:

`--dont-exit-on-errors` or `-e`: By default, the command line utility exits with a non-zero exit code when it encounters an error. If you pass this option, it will print the error message and continue running.
`--ignore-case` or `-i`: Allow all defined commit types to be uppercase as well as lowercase (e.g., "feat" and "FEAT").
`--types` or `-t`: This option allows you to override the commit types and the required fields for each commit type. For example, if you want to add a commit type named `feature` that requires a field named `scope`, you can pass the following argument: `--types "feature=scope"`. You can specify multiple commit types by separating them with semicolons, like this: `--types "fix=scope,description;feature=scope,body"`.

## Examples

Here are some examples of how you can use rcop:

To validate a commit message that has the type fix and the field scope:

```
echo "fix(scope): Some fixes" | rcop
```

To validate a commit message that has the type docs and no required fields:

```
echo "docs: Some updates to the documentation" | rcop --types "docs="
```

To validate a commit message that has the type docs and no required fields:

```
echo "wild(scope): Some updates to the documentation" | rcop --types "wild=scope,description"
```

To validate a commit message and print the error message instead of exiting:

```
echo "invalid: scope: Some invalid commit message" | rcop --dont-exit-on-errors
```

To validate a commit message that has an all caps commit type:

```
echo "DOCS: Some updates to the documentation" | rcop --ignore-case
```

## Output

When a commit message is successfully validated, rcop exits with a zero exit code and doesn't produce any output.

Otherwise, it prints an error message and exits with a non-zero exit code.
