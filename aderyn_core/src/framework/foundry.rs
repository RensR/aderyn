use crate::ast::*;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{canonicalize, read_dir, read_to_string, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Stdio;

// Foundry compiler output file
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundryOutput {
    pub ast: SourceUnit,
}

// Foundry TOML config file
#[derive(Debug, Deserialize)]
struct FoundryConfig {
    profile: ProfileSection,
}

#[derive(Debug, Deserialize)]
struct ProfileSection {
    #[serde(rename = "default")]
    default: DefaultProfile,
}

#[derive(Debug, Deserialize)]
struct DefaultProfile {
    #[serde(default = "default_src")]
    src: String,
    #[serde(default = "default_out")]
    out: String,
}

fn default_src() -> String {
    "src".to_string()
}

fn default_out() -> String {
    "out".to_string()
}

pub fn read_foundry_output_file(filepath: &str) -> Result<FoundryOutput> {
    Ok(serde_json::from_reader(BufReader::new(File::open(
        filepath,
    )?))?)
}

#[derive(Debug)]
pub struct LoadedFoundry {
    pub src_path: String,
    pub src_filepaths: Vec<PathBuf>,
    pub output_filepaths: Vec<PathBuf>,
}

// Load foundry and return a Vector of PathBufs to the AST JSON files
pub fn load_foundry(foundry_root: &PathBuf) -> Result<LoadedFoundry, Box<dyn Error>> {
    let foundry_root_absolute = canonicalize(foundry_root).unwrap_or_else(|err| {
        // Exit with a non-zero exit code
        eprintln!("Error getting absolute path of Foundry root directory");
        // print err
        eprintln!("{:?}", err);
        std::process::exit(1);
    });

    // Run `forge build` in the root
    let _output = std::process::Command::new("forge")
        .arg("build")
        .current_dir(&foundry_root_absolute)
        .stdout(Stdio::inherit()) // This will stream the stdout
        .stderr(Stdio::inherit())
        .status();

    let foundry_config_filepath = foundry_root_absolute.join("foundry.toml");
    let foundry_config = read_config(&foundry_config_filepath).unwrap_or_else(|_err| {
        // Exit with a non-zero exit code
        eprintln!("Error reading Foundry config file");
        std::process::exit(1);
    });

    // Get the file names of all contracts in the Foundry src directory
    let foundry_src_path = foundry_root_absolute.join(&foundry_config.profile.default.src);
    let contract_filepaths =
        collect_nested_files(&foundry_src_path, "sol").unwrap_or_else(|_err| {
            // Exit with a non-zero exit code
            eprintln!("Error collecting Solidity files from Foundry src directory");
            std::process::exit(1);
        });

    // For each contract in the Foundry output directory, check if it is in the list of contracts in the Foundry src directory
    // (This is because some contracts may be imported but not deployed, or there may be old contracts in the output directory)
    let foundry_out_path = foundry_root_absolute.join(&foundry_config.profile.default.out);

    let json_output_filepaths = collect_nested_files(&foundry_out_path.clone(), "json")
        .unwrap_or_else(|_err| {
            // Exit with a non-zero exit code
            eprintln!("Error collecting JSON output files from Foundry output directory");
            std::process::exit(1);
        });
    let output_filepaths = get_matching_output_files(&json_output_filepaths, &contract_filepaths);

    Ok(LoadedFoundry {
        src_path: foundry_config.profile.default.src,
        src_filepaths: contract_filepaths,
        output_filepaths,
    })
}

fn read_config(path: &PathBuf) -> Result<FoundryConfig, Box<dyn Error>> {
    let contents = read_to_string(path).unwrap();
    let foundry_config_toml = toml::from_str(&contents);
    let foundry_config = match foundry_config_toml {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing TOML: {:?}", e);
            std::process::exit(1);
        }
    };
    Ok(foundry_config)
}

fn collect_nested_files(path: &PathBuf, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut results = Vec::new();

    if path.is_dir() {
        for entry in read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                results.extend(collect_nested_files(&entry_path, extension)?);
            } else if entry_path.extension().map_or(false, |ext| ext == extension) {
                results.push(entry_path);
            }
        }
    } else if path.extension().map_or(false, |ext| ext == extension) {
        results.push(path.clone());
    }

    Ok(results)
}

fn get_matching_output_files(
    json_output_filepaths: &[PathBuf],
    src_filepaths: &[PathBuf],
) -> Vec<PathBuf> {
    json_output_filepaths
        .iter()
        .filter(|output_filepath| {
            src_filepaths.iter().any(|src_filepath| {
                let contract_name = src_filepath.file_name().unwrap().to_str().unwrap();
                output_filepath
                    .to_str()
                    .map_or(false, |s| s.contains(contract_name))
            })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use rayon::result;

    use super::*;

    #[test]
    fn test_nested_contracts_with_same_name() {
        let cargo_root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let tests_contract_playground_path = cargo_root
            .join("../tests/contract-playground/")
            .canonicalize()
            .unwrap();
        let result = load_foundry(&tests_contract_playground_path).unwrap();
        let nested_1_exists = result
            .output_filepaths
            .iter()
            .any(|path| path.to_str().unwrap().contains("1/Nested.sol"));
        let nested_2_exists = result
            .output_filepaths
            .iter()
            .any(|path| path.to_str().unwrap().contains("2/Nested.sol"));
        assert!(nested_1_exists);
        assert!(nested_2_exists);
    }
}
