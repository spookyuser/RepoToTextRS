# RepoToText

RepoToText is a Rust-based command-line tool that converts the structure and files of a repository into a text file probably for uploading to an llm.

Inspired by [this](https://github.com/JeremiahPetersen/RepoToText)

## Features

- Converts repository structure and files to text.
- Allows you to specify include and exclude glob patterns for files.
- Reads .gitignore file if present and excludes those files.
- Allows you to specify the output file path.
- If no output file path is provided, it generates a default one in the /tmp directory.
- Prints the directory structure using the `tree` command.

## Usage

Run the `repototext` command with the following arguments:

- `repo_path`: Path to the repository (required).
- `output_path`: Path to the output text file (optional).
- `package_name`: Name of the package to include in the output file headers (optional).
- `include`: Glob pattern for including files (e.g., '**/*.rs') (optional).
- `exclude`: Glob pattern for excluding files (e.g., 'node_modules/**') (optional).

Example:

```sh
cargo run -- repo_path output_path package_name --include '**/*.rs' --exclude 'node_modules/**'
```

## Environment Variables

You can also specify include and exclude glob patterns through the following environment variables:

- `REPOTOTEXT_INCLUDE_GLOBS`: Glob pattern for including files.
- `REPOTOTEXT_EXCLUDE_GLOBS`: Glob pattern for excluding files.
- `REPOTOTEXT_TREE_EXCLUDES`: Directories to exclude from the `tree` command.

## Building

To build the project, run:

```sh
cargo build
```

## License

This project is licensed under the MIT License.