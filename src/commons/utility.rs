use std::collections::HashSet;
use std::fs::{canonicalize, DirEntry};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use anyhow::{Error, Result};
use git2::build::RepoBuilder;
use git2::{FetchOptions, ProxyOptions};

use crate::display_control::display_message;
use crate::display_control::display_tree_message;
use crate::display_control::Level;
use crate::core::chain::Chain;
use crate::marker::bookmark::Bookmark;
use crate::marker::reference::TrackPath;

use super::errors::PackageError;
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

pub fn check_required_packages(chain: &(impl AvailablePackages + TrackPath)) -> Result<(), Error> {
    let required_packages: HashSet<Package> = chain.get_missing_packages()?;
    
    if !required_packages.is_empty() {
        return Err(PackageError::MissingPackages {
            number_missed_packages: required_packages.len(),
            missed_packages: required_packages.into_iter()
                .map(|p| format!("\n    - {}", p.access_package_name()))
                .collect(),
            chain_name: chain.get_path().to_string()
        }.into());
    }
    
    Ok(())
}

/// Handle the case in which the input string is a git repo.
/// This returns a local path to the cloned git repo. 
fn handle_remote_url(input_string: &str) -> Result<String, Error> {
    let current_dir: PathBuf = std::env::current_dir()?
        .canonicalize()?
        .join(input_string.split("/").last().unwrap());
    
    let mut fetch_options = FetchOptions::new();
    let mut proxy_options = ProxyOptions::new();
    proxy_options.auto();
    fetch_options.proxy_options(proxy_options);
    
    let repository = RepoBuilder::new()
        .fetch_options(
            fetch_options
        )
        .clone(input_string, &current_dir)?;
    
    return Ok(repository.workdir().unwrap().to_string_lossy().to_string());
}

pub fn handle_adding_bookmarks_logics(bookmark: &mut Bookmark, input_string: &str) -> Result<(), Error> {
    
    let path: PathBuf = if input_string.contains("github") {
        display_message(Level::Logging, "GitHub repository detected. Try adding bookmarks from there...");
        let local_path = PathBuf::from(handle_remote_url(input_string)?);
        display_message(Level::Logging, &format!("Repository cloned to: {}", local_path.display()));
        
        local_path
    } else {
        PathBuf::from(input_string)
    };
    
    let path = path.as_path();
    
    if !path.exists() {
        return Err(anyhow!("Provided path does not exist! Operation aborted."));
    }

    if path.is_dir() {
        let fullpath: std::path::PathBuf = canonicalize(&path)?;
        let filepaths: Vec<DirEntry> = get_paths(Path::new(&fullpath))?;
        display_message(
            Level::Logging,
            &format!("Registering {} chains to the bookmark", filepaths.len()),
        );
        for filepath in filepaths {
            // Acquire the canonical path of the file
            let path_string: String =filepath
                .path()
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .to_string();
            
            match bookmark.add_chain_reference(
                path_string
            ) {
                Ok(_) => {
                    display_message(
                        Level::Logging,
                        &format!(
                            "{} is registered successfully.",
                            filepath.path().canonicalize().unwrap().to_str().unwrap()
                        ),
                    );
                    continue;
                }
                Err(error) => {
                    match error.downcast_ref::<PackageError>() {
                        Some(
                            PackageError::MissingPackages { 
                                number_missed_packages: _, 
                                missed_packages: _, 
                                chain_name: _ 
                            }
                        ) => {
                            display_message(
                                Level::Warn,
                                &error.to_string()
                            );
                        }
                        _ => {
                            display_message(
                                Level::Warn,
                                &format!("{}, skipped bookmarking.", error.to_string()),
                            );
                        }
                    }
                    continue;
                }
            };
        }
        
        return Ok(());
    }

    if path.is_file() {
        display_message(Level::Logging, "Registering a chain to the bookmark");

        let filepath: &Path = Path::new(&path);
        let path_string: String = filepath
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();
        
        match bookmark.add_chain_reference(path_string) {
            Ok(_) => display_message(
                Level::Logging,
                &format!(
                    "{} is registered successfully.",
                    filepath.canonicalize().unwrap().to_str().unwrap()
                ),
            ),
            Err(error) => {
                match error.downcast_ref::<PackageError>() {
                    Some(
                        PackageError::MissingPackages { 
                            number_missed_packages: _, 
                            missed_packages: _, 
                            chain_name: _ 
                        }
                    ) => {
                        display_message(
                            Level::Warn,
                            &error.to_string()
                        );
                    }
                    _ => {
                        display_message(
                            Level::Warn,
                            &format!("{}, skipped bookmarking.", error.to_string()),
                        );
                    }
                }
            }
        };
        
        return Ok(());
    }
    
    Err(anyhow!("The specified path is not valid. Please check."))
}