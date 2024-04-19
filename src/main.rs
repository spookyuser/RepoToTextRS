use clap::{Arg, Command};
use figment::providers::Format;
use serde::Deserialize;

use figment::{providers::Toml, Figment};
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct Config {
    tree_exclude_globs: String,
    ignore_files: Vec<String>,
    file_extensions: Vec<String>,
}

fn main() -> Result<()> {
    let matches = Command::new("repototext")
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
            Arg::new("extensions")
                .short('e')
                .long("extensions")
                .value_name("EXT")
                .help("File extensions to include (comma-separated)")
                .use_value_delimiter(true)
                .value_delimiter(','),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .value_name("PATTERN")
                .help("Ignore files/directories matching the given pattern")
                .use_value_delimiter(true)
                .value_delimiter(','),
        )
        .get_matches();

    let repo_path = matches.get_one::<String>("repo_path").unwrap();

    // Load default configuration
    let mut figment = Figment::new();

    // Load configuration from config.toml in the user's home directory
    figment = figment.merge(Toml::file(
        "/Users/caleb/Developer/RepoToTestRS/config.toml",
    ));

    // Check if config.toml exists in the root of the repo directory
    let config_path = Path::new(repo_path).join("config.toml");
    if config_path.exists() {
        // Merge repository-specific configuration
        figment = figment.merge(Toml::file(config_path));
    }

    // Extract the merged configuration
    let config: Config = figment.extract().unwrap();

    // Override file extensions and ignore patterns from command line if provided
    let file_extensions = matches
        .get_many::<String>("extensions")
        .map(|values| values.map(|ext| ext.to_string()).collect())
        .unwrap_or(config.file_extensions);

    let ignore_files = matches
        .get_many::<String>("ignore")
        .map(|values| values.map(|pattern| pattern.to_string()).collect())
        .unwrap_or(config.ignore_files);

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

    // Use fd-find to generate file paths based on file extensions
    let mut fd_command = ProcessCommand::new("fd");
    fd_command.arg("--type").arg("file").arg("--unrestricted");

    // Add file extensions
    for extension in &file_extensions {
        fd_command.arg("-e").arg(extension);
    }

    fd_command.arg(".").arg(repo_path);

    // Exclude paths based on ignore patterns
    for ignore_pattern in &ignore_files {
        fd_command.arg("--exclude").arg(ignore_pattern);
    }

    let fd_output = fd_command.output()?;
    let file_paths = String::from_utf8_lossy(&fd_output.stdout);

    // Process each file path
    for path in file_paths.lines() {
        let path = Path::new(path);
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
                prefix = "",
                path = path.display()
            )?;
            writeln!(output_file, "{}", content)?;
            writeln!(
                output_file,
                "===== END {prefix}/{path} =====\n\n",
                prefix = "",
                path = path.display()
            )?;
        }
    }

    // Run tree command from the command line
    let output = ProcessCommand::new("tree")
        .arg(repo_path)
        .arg("-I")
        .arg(config.tree_exclude_globs)
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
    println!("FD Command: {:?}", fd_command);
    println!("Output saved to: {}", output_path);
    println!("Include extensions: {:?}", file_extensions);
    println!("Ignore patterns: {:?}", ignore_files);
    println!("FD Output: {}", file_paths);

    let output_file_parent = Path::new(&output_path).parent().unwrap();

    match ProcessCommand::new("open").arg(output_file_parent).output() {
        Ok(_report) => {}
        Err(_err) => {}
    }

    Ok(())
}
