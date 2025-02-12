use std::fs::{canonicalize, DirEntry};
use std::path::Path;

use anyhow::{Error, Result};
use cchain::arguments::*;
use cchain::cli::traits::Execution;
use cchain::commons::naming::HumanReadable;
use cchain::display_control::{display_form, display_message, Level};
use cchain::commons::utility::{
    configuration_selection, generate_template, get_paths, resolve_cchain_configuration_filepaths
};
use cchain::marker::reference::ChainReference;
use cchain::{marker::bookmark::Bookmark, chain::Chain};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse command line arguments
    let mut arguments = Arguments::parse();
    // Instantiate the bookmark
    let mut bookmark: Bookmark = Bookmark::from_file();

    // Map the arguments to corresponding code logics
    match arguments.commands {
        Commands::Run(subcommand) => {
            match subcommand.chain {
                Some(path) => {
                    // Load and parse the configuration file
                    let mut chain: Chain = Chain::from_file(&path)?;
                    // Iterate over each configuration and execute the commands
                    chain.execute().await?;
                },
                None => {
                    display_message(
                        Level::Error,
                        "Please provide a chain path to run"
                    );
                }
            }
        },
        Commands::Add(subcommand) => {
            if let Some(path) = subcommand.path {
                display_message(
                    Level::Logging,
                    "Registering multiple configuration file paths to the bookmark"
                );
                let filepaths: Vec<DirEntry> = get_paths(Path::new(&path));
                for filepath in filepaths {
                    match bookmark.add_chain_reference(
                        filepath.path()
                            .canonicalize()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()) {
                        Ok(_) => {
                            display_message(
                                Level::Logging,
                                &format!(
                                    "{} is registered successfully.", 
                                    filepath.path()
                                        .canonicalize()
                                        .unwrap()
                                        .to_str()
                                        .unwrap())
                            );
                            continue;
                        }
                        Err(error) => {
                            display_message(
                                Level::Warn, 
                                &format!("{}, skipped bookmarking.", error.to_string())
                            );
                            continue;
                        }
                    };
                }

                display_message(
                    Level::Logging, 
                    "Bookmark registration is done."
                );
                bookmark.save();
                return Ok(());
            }
        },
        Commands::List(subcommand) => {
            let references: &Vec<ChainReference> = &bookmark.get_chain_references();
            let mut form_data: Vec<Vec<&str>>;

            for reference in references {
                form_data.push(
                    vec![
                        &reference.get_chain_path_string(),
                        &reference.get_human_readable_name()
                    ]
                );
            }

            display_form(
                vec!["Name", "Path"], 
                form_data
            );
        },
        Commands::Remove(subcommand) => {},
        Commands::Generate(subcommand) => {
            if subcommand.llm {
                display_message(
                    Level::Error, 
                    "LLM generation feature has not yet implemented. Stay tuned. 😈"
                );
            }
            generate_template(subcommand.name.as_deref());

            return Ok(());
        }
    }

    // Convert the relative path into absolute for configuration_file
    if let Some(path) = arguments.configuration_file {
        arguments.configuration_file = Some(canonicalize(path)?.to_string_lossy().to_string());
    }

    // If `configuration_files` is set, get the file paths first.
    let configuration_filepaths: Option<Vec<String>> =
        if let Some(files_path) = &arguments.configuration_files {
            // Resolve the file paths from the provided directory
            match resolve_cchain_configuration_filepaths(std::path::Path::new(files_path)) {
                Ok(filepaths) => Some(filepaths),
                Err(e) => {
                    display_message(
                        Level::Error, &e.to_string()
                    );
                    std::process::exit(1);
                }
            }
        } else {
            None
        };

    if arguments.delete_bookmark {
        let bookmarked_configuration_filepaths: &Vec<String> =
            bookmark.get_bookmarked_configurations();

        let selected_configuration: String =
            configuration_selection(bookmarked_configuration_filepaths.to_vec());

        bookmark.unbookmark_configuration_by_path(&selected_configuration)?;

        bookmark.save();

        display_message(
            Level::Warn, 
            &format!(
                "Bookmark at {} is removed from the collection.",
                selected_configuration
            )
        );

        return Ok(());
    }

    // If neither configuration_file nor configuration_files is set, prompt the user to select from bookmarked configurations
    if arguments.configuration_file.is_none() && arguments.configuration_files.is_none() {
        let bookmarked_configuration_filepaths = bookmark.get_bookmarked_configurations();
        arguments.configuration_file = Some(configuration_selection(
            bookmarked_configuration_filepaths.to_vec(),
        ));
    }

    // Check if the bookmark flag is set, and if so, register the filepath to the bookmark.
    // We first check if the configuration_file or configuration_files flag is set,
    // and an error will be thrown if both are set.
    if arguments.bookmark {
        // Ensure that both configuration_file and configuration_files flags are not set simultaneously
        if arguments.configuration_file.is_some() && arguments.configuration_files.is_some() {
            display_message(
                Level::Error, 
                "Cannot set both configuration_file and configuration_files flags"
            );
            return Err(Error::msg(
                "Cannot set both configuration_file and configuration_files flags",
            ));
        }

        // Register the single configuration file path to the bookmark if configuration_file is set
        if let Some(filepath) = &arguments.configuration_file {
            display_message(
                Level::Logging, 
                &format!(
                    "Registering single configuration file path to the bookmark: {}",
                    filepath
                )
            );
            bookmark.bookmark_configuration(filepath.clone())?;
        }
        // Register multiple configuration file paths to the bookmark if configuration_files is set
        else if let Some(filepaths) = configuration_filepaths {}

    }

    // Prompt the user for selecting a configuration file to execute,
    // if any.
    let configurations_file: String = if let Some(filepath) = arguments.configuration_file {
        filepath
    } else {
        if let Some(filepaths) = configuration_filepaths {
            configuration_selection(filepaths)
        } else {
            display_message(
                Level::Error, 
                "No configuration file or file paths provided"
            );
            std::process::exit(1);
        }
    };

    Ok(())
}
