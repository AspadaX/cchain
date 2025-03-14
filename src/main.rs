use std::fs::{canonicalize, DirEntry};
use std::path::Path;

use anyhow::{Error, Result};
use cchain::arguments::*;
use cchain::core::traits::Execution;
use cchain::commons::naming::HumanReadable;
use cchain::commons::utility::{get_paths, input_message};
use cchain::display_control::{display_form, display_message, display_tree_message, Level};
use cchain::generations::create::ChainCreation;
use cchain::marker::reference::ChainReference;
use cchain::{core::chain::Chain, marker::bookmark::Bookmark};
use clap::{crate_version, Parser};

fn main() -> Result<(), Error> {
    // Parse command line arguments
    let arguments = Arguments::parse();
    // Instantiate the bookmark
    let mut bookmark = Bookmark::from_file();

    // Map the arguments to corresponding code logics
    match arguments.commands {
        Commands::Run(subcommand) => {
            // If the input is parsable into an usize, it will use it as an
            // index to the bookmark. Otherwise, it will use it as a path
            match subcommand.chain.parse::<usize>() {
                Ok(index) => {
                    if let Some(chain_reference) = bookmark.get_chain_reference_by_index(index) {
                        let mut chain = Chain::from_file(&chain_reference.get_chain_path_string())?;
                        chain.execute()?;
                    }
                }
                Err(_) => {
                    let mut chain: Option<Chain> = None;
                    // If the input is a path to a chain 
                    let path = Path::new(&subcommand.chain);
                    if path.exists() && path.is_file() {
                        if let Some(extension) = path.extension() {
                            if extension == "json" {
                                if let Some(file_name) = path.file_name() {
                                    if file_name.to_string_lossy().starts_with("cchain_") {
                                        // Load and parse the configuration file
                                        chain = Some(Chain::from_file(&subcommand.chain)?);
                                    }
                                }
                            }
                        }
                    } else {
                        // If the input is keywords
                        let result = bookmark.get_chains_by_keywords(
                            subcommand.chain
                                .split_whitespace()
                                .map(String::from)
                                .collect::<Vec<String>>()
                        );
                        
                        if let Some(chain_references) = result {
                            // Throw an error if no chains are found
                            if chain_references.len() == 0 {
                                display_message(Level::Error, "No chains found");
                                return Ok(());
                            }
                            
                            // Run the chain if it is exactly one
                            if chain_references.len() == 1 {
                                chain = Some(Chain::from_file(&chain_references[0].get_chain_path_string())?);
                            } else {
                                // Provide selections if multiple chains are found
                                display_message(Level::Logging, "Multiple chains found:");
                                for (index, chain_reference) in chain_references.iter().enumerate() {
                                    display_tree_message(1, &format!("{}: {}", index + 1, chain_reference.get_human_readable_name()));
                                }
                                let selection: usize = input_message("Please select a chain to execute:")?.trim().parse::<usize>()?;
                                chain = Some(Chain::from_file(&chain_references[selection - 1].get_chain_path_string())?);
                            }
                        }
                    }
                    
                    // Iterate over each configuration and execute the commands
                    if let Some(mut chain) = chain {
                        match chain.execute() {
                            Ok(_) => return Ok(()),
                            Err(_) => {
                                chain.show_statistics();
                                display_message(
                                    Level::Error,
                                    "Chain execution finished with error(s) occurred",
                                );
                            }
                        };
                    } else {
                        display_message(Level::Error, "No chain to execute");
                    }
                }
            }
        },
        Commands::Add(subcommand) => {
            let path = Path::new(&subcommand.path);

            if !path.exists() {
                display_message(
                    Level::Error,
                    &format!("Provided path does not exist! Operation aborted."),
                );
            }

            if path.is_dir() {
                let fullpath = canonicalize(&path)?;
                let filepaths: Vec<DirEntry> = get_paths(Path::new(&fullpath))?;
                display_message(
                    Level::Logging,
                    &format!("Registering {} chains to the bookmark", filepaths.len()),
                );
                for filepath in filepaths {
                    match bookmark.add_chain_reference(
                        filepath
                            .path()
                            .canonicalize()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
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
                            display_message(
                                Level::Warn,
                                &format!("{}, skipped bookmarking.", error.to_string()),
                            );
                            continue;
                        }
                    };
                }
            }

            if path.is_file() {
                display_message(Level::Logging, "Registering a chain to the bookmark");

                let filepath: &Path = Path::new(&path);

                match bookmark.add_chain_reference(
                    filepath
                        .canonicalize()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                ) {
                    Ok(_) => display_message(
                        Level::Logging,
                        &format!(
                            "{} is registered successfully.",
                            filepath.canonicalize().unwrap().to_str().unwrap()
                        ),
                    ),
                    Err(error) => {
                        display_message(
                            Level::Warn,
                            &format!("{}, skipped bookmarking.", error.to_string()),
                        );
                    }
                };
            }

            display_message(Level::Logging, "Bookmark registration is done.");
            bookmark.save();
            return Ok(());
        },
        Commands::List(_) => {
            let references: &Vec<ChainReference> = &bookmark.get_chain_references();
            let mut form_data: Vec<Vec<String>> = Vec::new();

            for (index, reference) in references.iter().enumerate() {
                form_data.push(vec![
                    index.to_string(),
                    reference.get_human_readable_name(),
                    reference.get_chain_path_string(),
                ]);
            }

            display_form(vec!["Index", "Name", "Path"], &form_data);
        },
        Commands::Remove(subcommand) => {
            if subcommand.reset {
                Bookmark::reset()?;
                display_message(Level::Warn, "Bookmark has been reset!");
            } else {
                if let Some(index) = subcommand.index {
                    let reference = match bookmark.get_chain_reference_by_index(index) {
                        Some(result) => result,
                        None => {
                            display_message(
                                Level::Error,
                                &format!("Bookmark index {} is not found", index),
                            );
                            return Ok(());
                        }
                    };
                    let reference_name: String = reference.get_chain_path_string();
                    bookmark.remove_chain_reference_by_index(index)?;
                    bookmark.save();

                    display_message(
                        Level::Warn,
                        &format!(
                            "Bookmark at {} is removed from the collection.",
                            &reference_name
                        ),
                    );
                }
            }

            return Ok(());
        },
        Commands::Clean(_) => {
            let invalid_paths: Vec<String> = bookmark.get_invalid_paths()?;
            let mut cleaned_invalid_paths: usize = 0;

            if invalid_paths.len() == 0 {
                display_message(
                    Level::Logging, 
                    "No chains need to be cleaned. All good! 😎"
                );

                return Ok(());
            }

            for invalid_path in invalid_paths {
                match bookmark.remove_chain_reference_by_path(&invalid_path) {
                    Ok(_) => {
                        cleaned_invalid_paths += 1;
                        display_message(
                            Level::Logging, 
                            &format!(
                                "Chain does no longer exist at: {}, cleaned.", 
                                &invalid_path
                            )
                        );
                    },
                    Err(error) => display_message(
                        Level::Error, 
                        &format!(
                            "Error has occurred when trying removing chain at: {} 😥", 
                            error
                        )
                    ),
                };
            }

            display_message(
                Level::Logging, 
                &format!(
                    "{} invalid chains paths are cleaned from the bookmark.", 
                    cleaned_invalid_paths
                )
            );
            bookmark.save();
        },
        Commands::Check(subcommand) => {
            // If the input is parsable into an usize, it will use it as an
            // index to the bookmark. Otherwise, it will use it as a path
            match subcommand.chain.parse::<usize>() {
                Ok(index) => {
                    if let Some(chain_reference) = bookmark.get_chain_reference_by_index(index) {
                        let mut chain = Chain::from_file(&chain_reference.get_chain_path_string())?;
                        chain.validate_syntax()?;
                    }
                }
                Err(_) => {
                    // Load and parse the configuration file
                    let mut chain = Chain::from_file(&subcommand.chain)?;
                    chain.validate_syntax()?;
                }
            }
        },
        Commands::New(subcommand) => {
            let result: String;
            let creation = ChainCreation::new(subcommand.name);
            display_message(
                Level::Logging,
                &format!(
                    "{} will be created...", creation.create_filename()
                )
            );
            
            if let Some(prompt) = subcommand.prompt {
                result = creation.generate_chain(
                    prompt
                )?;
            } else {
                result = creation.generate_template()?;
            }

            creation.save(result)?;
            return Ok(());
        },
        Commands::Version(_) => {
            display_message(
                Level::Logging,
                &format!("cchain version: {}", crate_version!()),
            );

            return Ok(());
        }
    }

    Ok(())
}
