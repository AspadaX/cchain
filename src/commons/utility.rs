use std::collections::HashSet;
use std::fs::DirEntry;
use std::io::Write;
use std::path::Path;

use anyhow::anyhow;
use anyhow::{Error, Result};

use crate::display_control::display_message;
use crate::display_control::display_tree_message;
use crate::display_control::Level;
use crate::core::chain::Chain;
use crate::marker::bookmark::Bookmark;

use super::naming::HumanReadable;
use super::packages::{AvailablePackages, Package};

pub fn get_paths(path: &std::path::Path) -> Result<Vec<DirEntry>, Error> {
    let mut paths: Vec<DirEntry> = Vec::new();
    let entries = std::fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        if entry.path().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "json")
            && entry.file_name().to_string_lossy().starts_with("cchain_")
        {
            paths.push(entry);
        }
    }
    Ok(paths)
}

pub fn input_message(prompt: &str) -> Result<String, Error> {
    // display the prompt message for inputting values
    display_message(Level::Input, prompt);
    // collect the input as a string
    let mut input = String::new();
    // receive stdin
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;

    Ok(input)
}

/// Resolve a path into a chain
pub fn read_into_chain(input_string: &str, bookmark: &Bookmark) -> Result<Chain, Error> {
    let path = Path::new(input_string);
    
    // Determine if the input is a valid chain file
    if path.exists() && path.is_file() {
        if let Some(extension) = path.extension() {
            if extension == "json" {
                if let Some(file_name) = path.file_name() {
                    if file_name.to_string_lossy().starts_with("cchain_") {
                        // Load and parse the configuration file
                        return Ok(Chain::from_file(input_string)?);
                    }
                }
            }
        }
    }
    
    // If the input is keywords
    let result = bookmark.get_chains_by_keywords(
        input_string
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<String>>()
    );
    
    if let Some(chain_references) = result {
        // Throw an error if no chains are found
        if chain_references.len() == 0 {
            return Err(anyhow!("No chains found"));
        }
        
        // Run the chain if it is exactly one
        if chain_references.len() == 1 {
            return Ok(Chain::from_file(&chain_references[0].get_chain_path_string())?);
        }
        
        // Provide selections if multiple chains are found
        display_message(Level::Logging, "Multiple chains found:");
        for (index, chain_reference) in chain_references.iter().enumerate() {
            display_tree_message(1, &format!("{}: {}", index + 1, chain_reference.get_human_readable_name()));
        }
        let selection: usize = input_message("Please select a chain to execute:")?.trim().parse::<usize>()?;
        
        return Ok(Chain::from_file(&chain_references[selection - 1].get_chain_path_string())?);
    }
    
    Err(anyhow!("No chains found"))
}

pub fn check_required_packages(chain: &Chain) -> Result<(), Error> {
    let required_packages: HashSet<Package> = chain.get_missing_packages()?;
    
    if !required_packages.is_empty() {
        let mut error_message: String = format!(
            "{} required packages are missing. Please install the following packages:", 
            required_packages.len()
        );
        for package in required_packages {
            error_message.push_str(&format!("\n     - {}", package.access_package_name()));
        }
        
        return Err(anyhow!(error_message));
    }
    
    Ok(())
}
