mod bookmark;
mod command;
mod configuration;
mod function;
mod utility;

use anyhow::{Error, Result};
use bookmark::Bookmark;
use clap::Parser;
use command::Arguments;
use configuration::Configuration;
use log::{error, info};
use utility::{
    configuration_selection, execute_argument_function, execute_command, generate_template,
    resolve_cchain_configuration_filepaths,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Setup a logger
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    // Parse command line arguments
    let mut arguments = Arguments::parse();

    // Instantiate the bookmark
    let mut bookmark: Bookmark = bookmark::Bookmark::from_file();

    // If `configuration_files` is set, get the file paths first.
    let configuration_filepaths: Option<Vec<String>> =
        if let Some(files_path) = &arguments.configuration_files {
            // Resolve the file paths from the provided directory
            match resolve_cchain_configuration_filepaths(std::path::Path::new(files_path)) {
                Ok(filepaths) => Some(filepaths),
                Err(e) => {
                    error!("{}", e);
                    std::process::exit(1);
                }
            }
        } else {
            None
        };

    // Check if the generate flag is set, and if so, generate a template configuration file
    if arguments.generate {
        generate_template();
        return Ok(());
    }

    // If neither configuration_file nor configuration_files is set, prompt the user to select from bookmarked configurations
    if arguments.configuration_file.is_none() && arguments.configuration_files.is_none() {
        let bookmarked_configuration_filepaths = bookmark.get_bookmarked_configurations();
        arguments.configuration_file = Some(configuration_selection(
            bookmarked_configuration_filepaths.to_vec(),
        ));
    }

    if arguments.delete_bookmark {
        let bookmarked_configuration_filepaths = bookmark.get_bookmarked_configurations();
        let selected_configuration =
            configuration_selection(bookmarked_configuration_filepaths.to_vec());
        bookmark.unbookmark_configuration_by_path(
            &selected_configuration
        )?;
        bookmark.save();
        return Ok(());
    }

    // Check if the bookmark flag is set, and if so, register the filepath to the bookmark.
    // We first check if the configuration_file or configuration_files flag is set,
    // and an error will be thrown if both are set.
    if arguments.bookmark {
        // Ensure that both configuration_file and configuration_files flags are not set simultaneously
        if arguments.configuration_file.is_some() && arguments.configuration_files.is_some() {
            error!("Cannot set both configuration_file and configuration_files flags");
            return Err(Error::msg(
                "Cannot set both configuration_file and configuration_files flags",
            ));
        }

        // Register the single configuration file path to the bookmark if configuration_file is set
        if let Some(filepath) = &arguments.configuration_file {
            info!(
                "Registering single configuration file path to the bookmark: {}",
                filepath
            );
            bookmark.bookmark_configuration(filepath.clone())?;
        }
        // Register multiple configuration file paths to the bookmark if configuration_files is set
        else if let Some(filepaths) = configuration_filepaths {
            info!("Registering multiple configuration file paths to the bookmark");
            for filepath in filepaths {
                info!("Registering configuration file path: {}", filepath);
                bookmark.bookmark_configuration(filepath)?;
            }
        }

        info!("Bookmark registration is done.");
        bookmark.save();
        return Ok(());
    }

    // Prompt the user for selecting a configuration file to execute,
    // if any.
    let configurations_file: String = if let Some(filepath) = arguments.configuration_file {
        filepath
    } else {
        if let Some(filepaths) = configuration_filepaths {
            configuration_selection(filepaths)
        } else {
            error!("No configuration file or file paths provided");
            std::process::exit(1);
        }
    };

    // Load and parse the configuration file
    let configurations: Vec<Configuration> = serde_json::from_str(
        &std::fs::read_to_string(&configurations_file).expect("Failed to load configurations"),
    )?;

    // Iterate over each configuration and execute the commands
    for mut configuration in configurations {
        execute_argument_function(&mut configuration).await?;
        execute_command(&configuration);
    }

    Ok(())
}
