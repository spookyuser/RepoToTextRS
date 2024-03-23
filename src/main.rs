use clap::{Arg, Command};

use std::fs::File;
use std::io::{Result, Write};
use walkdir::WalkDir;

// Configuration
const STANDARD_EXCLUSIONS: [&str; 6] = [
    "node_modules/",
    ".git/",
    ".DS_Store",
    "dist/",
    "build/",
    "*.log",
];
const CUSTOM_INCLUDES: [&str; 3] = ["package.json", "../../README.md", "README.md"];
const EXTENSIONS: [&str; 2] = [".ts", ".tsx"];

fn main() -> Result<()> {
    // Define command-line arguments with clap
    let matches = Command::new("repototext")
        .version("0.1.0")
        .author("Your Name")
        .about("Converts repository structure and files to text")
        .arg(
            Arg::new("repo_path")
                .required(true)
                .index(1)
                .help("Path to the repository"),
        )
        .arg(
            Arg::new("output_path")
                .required(true)
                .index(2)
                .help("Path to the output text file"),
        )
        .arg(
            Arg::new("package_name")
                .index(3)
                .required(false)
                .help("Name of the package to include in the output file headers"),
        )
        .get_matches();

    let default_package_name = "".to_string();
    let repo_path = matches.get_one::<String>("repo_path").unwrap();
    let output_path = matches.get_one::<String>("output_path").unwrap();
    let package_name = matches
        .get_one::<String>("package_name")
        .unwrap_or(&default_package_name);

    // Create output file
    let mut output_file = File::create(output_path)?;

    // Walk the directory structure and print to output file
    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_included(e.path()))
    {
        let path = entry.path();
        if path.is_file() {
            // Read and write file content
            let content = std::fs::read_to_string(path)?;

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

    let output = std::process::Command::new("tree").arg(repo_path).output();
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

    Ok(())
}

// Function to check if a file should be included
fn is_included(path: &std::path::Path) -> bool {
    let path_str = path.to_str().unwrap_or("");

    // Check for exclusions
    if STANDARD_EXCLUSIONS
        .iter()
        .any(|exclusion| path_str.contains(exclusion))
    {
        return false;
    }

    // Check for inclusions or matching extensions
    CUSTOM_INCLUDES
        .iter()
        .any(|inclusion| path_str.contains(inclusion))
        || EXTENSIONS.iter().any(|ext| path_str.ends_with(ext))
}
