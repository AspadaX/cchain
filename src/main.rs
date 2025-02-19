use std::fs::{canonicalize, DirEntry};
use std::path::Path;

use anyhow::{Error, Result};
use cchain::arguments::*;
use cchain::core::traits::Execution;
use cchain::commons::naming::HumanReadable;
use cchain::commons::utility::{generate_template, get_paths};
use cchain::display_control::{display_form, display_message, Level};
use cchain::marker::reference::ChainReference;
use cchain::{core::chain::Chain, marker::bookmark::Bookmark};
use clap::Parser;

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
                    // Load and parse the configuration file
                    let mut chain = Chain::from_file(&subcommand.chain)?;
                    // Iterate over each configuration and execute the commands
                    match chain.execute() {
                        Ok(_) => return Ok(()),
                        Err(_) => {
                            chain.show_statistics();
                            display_message(
                                Level::Error,
                                "Chain execution finished with error(s) ocurred",
                            );
                        }
                    };
                }
            }
        }
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
        }
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
        }
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
        }
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
        }
        Commands::New(subcommand) => {
            generate_template(subcommand.name.as_deref())?;

            return Ok(());
        }
        Commands::Generate(_) => {
            display_message(
                Level::Error,
                "LLM generation feature has not yet implemented. Stay tuned. 😈",
            );

            return Ok(());
        }
    }

    Ok(())
}
