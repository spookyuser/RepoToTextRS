use clap::{Arg, Command};
use glob_match;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

fn main() -> Result<()> {
    let tree_excludes = env::var("REPOTOTEXT_TREE_EXCLUDES").unwrap_or_default();
    let base_include_globs: Vec<String> = env::var("REPOTOTEXT_INCLUDE_GLOBS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let base_exclude_globs: Vec<String> = env::var("REPOTOTEXT_EXCLUDE_GLOBS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let matches = Command::new("repo_to_text")
        .version("0.1.0")
        .about("Converts repository structure and files to text")
        .arg(
            Arg::new("repo_path")
                .required(true)
                .index(1)
                .help("Path to the repository"),
        )
        .arg(
            Arg::new("output_path")
                .index(2)
                .required(false)
                .help("Path to the output text file"),
        )
        .arg(
            Arg::new("package_name")
                .index(3)
                .required(false)
                .help("Name of the package to include in the output file headers"),
        )
        .arg(
            Arg::new("include")
                .short('I')
                .long("include")
                .value_name("INCLUDE_GLOB")
                .help("Glob pattern for including files (e.g., '**/*.rs')")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .num_args(1..)
                .default_value("**"),
        )
        .arg(
            Arg::new("exclude")
                .short('e')
                .long("exclude")
                .value_name("EXCLUDE_GLOB")
                .help("Glob pattern for excluding files (e.g., 'node_modules/**')")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .num_args(1..),
        )
        .after_help(
            "Glob Syntax Help:
  - '*' matches any sequence of non-separator characters
  - '**' matches any sequence of characters, including separator characters
  - '?' matches any single non-separator character
  - '[...]' matches any single character inside the brackets
  - '{...}' matches any of the comma-separated patterns inside the braces",
        )
        .get_matches();

    let default_package_name = "".to_string();
    let repo_path = matches.get_one::<String>("repo_path").unwrap();
    let package_name = matches
        .get_one::<String>("package_name")
        .unwrap_or(&default_package_name);

    // Parse include and exclude glob patterns from command-line arguments
    let cli_include_globs: Vec<String> = matches
        .get_many::<String>("include")
        .unwrap_or_default()
        .cloned()
        .collect();
    let cli_exclude_globs: Vec<String> = matches
        .get_many::<String>("exclude")
        .unwrap_or_default()
        .cloned()
        .collect();

    // Read .gitignore file if present
    let gitignore_path = Path::new(repo_path).join(".gitignore");
    let gitignore_globs: Vec<String> = if gitignore_path.exists() {
        let file = File::open(gitignore_path)?;
        let reader = BufReader::new(file);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .collect()
    } else {
        Vec::new()
    };

    // Combine glob patterns from environment variables, command-line arguments, and .gitignore
    let merged_include_globs: Vec<String> = base_include_globs
        .into_iter()
        .chain(cli_include_globs.into_iter())
        .collect();
    let merged_exclude_globs: Vec<String> = base_exclude_globs
        .into_iter()
        .chain(cli_exclude_globs.into_iter())
        .chain(gitignore_globs.into_iter())
        .collect();

    // Generate default output file path if not provided
    let output_path = match matches.get_one::<String>("output_path") {
        Some(path) => path.to_owned(),
        None => {
            let epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Failed to get epoch time");
            format!("/tmp/repo_to_text_{}.txt", epoch.as_millis())
        }
    };

    // Create output file
    let mut output_file = File::create(&output_path)?;

    // Walk the directory structure and print to output file
    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_included(e.path(), &merged_exclude_globs, &merged_include_globs))
    {
        let path = entry.path();
        if path.is_file() {
            // Read and write file content
            let content = match std::fs::read_to_string(path) {
                Ok(content) => content,
                Err(_) => {
                    continue;
                }
            };

            writeln!(
                output_file,
                "===== BEGIN {prefix}/{path} =====",
                prefix = package_name,
                path = path.display()
            )?;
            writeln!(output_file, "{}", content)?;
            writeln!(
                output_file,
                "===== END {prefix}/{path} =====\n\n",
                prefix = package_name,
                path = path.display()
            )?;
        }
    }

    // Run tree command from the command line
    let output = std::process::Command::new("tree")
        .arg(repo_path)
        .arg("-I")
        .arg(tree_excludes)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            writeln!(output_file, "===== BEGIN TREE =====")?;
            writeln!(output_file, "{}", stdout)?;
            writeln!(output_file, "===== END TREE =====\n\n")?;
        }
        Err(e) => {
            eprintln!("Error running tree command: {}", e);
        }
    }

    println!("Output saved to: {}", output_path);
    let output_file_parent = Path::new(&output_path).parent().unwrap();

    match std::process::Command::new("open")
        .arg(output_file_parent)
        .output()
    {
        Ok(report) => {}
        Err(err) => {}
    }

    Ok(())
}

// Function to check if a file should be included
fn is_included(path: &std::path::Path, exclude_globs: &[String], include_globs: &[String]) -> bool {
    let path_str = path.to_str().unwrap_or("");

    // Check for exclusions
    if exclude_globs
        .iter()
        .any(|glob| glob_match::glob_match(glob, path_str))
    {
        return false;
    }

    // Check for inclusions
    include_globs.is_empty()
        || include_globs
            .iter()
            .any(|glob| glob_match::glob_match(glob, path_str))
}
