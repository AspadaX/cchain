mod configuration;
mod function;

use std::fs::DirEntry;
use std::str::FromStr;

use anyhow::{Error, Result};
use clap::Parser;
use configuration::Configuration;
use log::{error, info};

#[derive(Parser)]
pub struct Arguments {
    // path to the command line chain configuration file
    #[clap(short, long)]
    pub configuration_file: Option<String>,

    // path to the directory containing the command line chain configuration files
    #[clap(short = 'd', long)]
    pub configuration_files: Option<String>,

    // generate a configuration template
    #[clap(short, long)]
    pub generate: bool,
}

fn get_paths(path: &std::path::Path) -> Vec<DirEntry> {
    let mut paths: Vec<DirEntry> = Vec::new();
    let entries = std::fs::read_dir(path).expect("Failed to read directory");
    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        if entry.path().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "json")
            && entry.file_name().to_string_lossy().starts_with("cchain_")
        {
            paths.push(entry);
        }
    }
    paths
}

pub fn determine_configuration_file(
    path_to_configurations: Option<&std::path::Path>,
    arguments: &Arguments,
) -> String {
    let paths: Vec<DirEntry> = if let Some(path) = path_to_configurations {
        get_paths(path)
    } else {
        get_paths(std::path::Path::new("."))
    };

    if let Some(config_file) = &arguments.configuration_file {
        config_file.clone()
    } else {
        // If no configuration files are found, log an error and return
        if paths.is_empty() {
            error!("No configuration files found starting with 'cchain_'");
            std::process::exit(1);
        }

        // List available configuration files for the user to select
        println!("\nAvailable configuration files:");
        for (i, path) in paths.iter().enumerate() {
            println!("{}: {}", i, path.file_name().to_string_lossy());
        }

        // Prompt the user to select a configuration file
        println!(
            "\nPlease select a configuration file to execute by entering the corresponding number:"
        );
        let mut selection = String::new();
        std::io::stdin()
            .read_line(&mut selection)
            .expect("Failed to read input");
        let index: usize = selection.trim().parse().expect("Invalid selection");

        // Return the selected configuration file path
        paths[index].path().to_string_lossy().to_string()
    }
}

pub fn execute_command(configuration: &Configuration) {
    // Create a new command based on the configuration
    let mut command = std::process::Command::new(configuration.get_command());
    // Add the arguments to the command
    command.args(configuration.get_arguments());

    // Spawn the command as a child process
    let mut child = command.spawn().expect("Failed to execute command");
    // Wait for the child process to finish
    let status = child.wait().expect("Failed to wait on child");

    // If the command failed and retry is enabled, try to execute it again
    let mut attempts = 0;
    while !status.success()
        && (configuration.get_retry() == &-1 || &attempts < configuration.get_retry())
    {
        attempts += 1;
        info!(
            "\nRetrying command: {}, attempt: {}",
            configuration, attempts
        );
        // Spawn the command again as a child process
        let status = command
            .spawn()
            .expect("Failed to execute command")
            .wait()
            .expect("Failed to wait on child");
        // If the command fails again and retry limit is reached, print an error message and stop the chain
        if !status.success()
            && configuration.get_retry() != &-1
            && &attempts >= configuration.get_retry()
        {
            error!("\nFailed to execute command: {}", configuration);
            return;
        }
    }

    // If the command fails and retry is -1, keep retrying indefinitely
    if !status.success() && configuration.get_retry() == &-1 {
        loop {
            attempts += 1;
            info!(
                "\nRetrying command: {}, attempt: {}",
                configuration, attempts
            );
            let status = command
                .spawn()
                .expect("Failed to execute command")
                .wait()
                .expect("Failed to wait on child");
            if status.success() {
                break;
            }
        }
    }

    // If the command fails and retry is 0, stop the chain
    if !status.success() && configuration.get_retry() == &0 {
        error!("\nFailed to execute command: {}\n", configuration);
        return;
    }

    // Separation between commands
    info!("\nFinished executing command: {}\n", configuration);
}

pub fn generate_template() {
    // Create a template configuration
    let template = vec![
        Configuration::new(
            "example_command".to_string(),
            vec!["arg1".to_string(), "arg2".to_string()],
            3,
        ),
        Configuration::new(
            "another_command".to_string(),
            vec!["argA".to_string(), "argB".to_string()],
            5,
        ),
    ];
    // Serialize the template to JSON
    let template_json =
        serde_json::to_string_pretty(&template).expect("Failed to serialize template");
    // Write the template JSON to a file
    std::fs::write("cchain_template.json", template_json).expect("Failed to write template file");
    info!("Template configuration file generated: cchain_template.json");
}

pub async fn execute_argument_function(configuration: &mut Configuration) -> Result<(), Error> {
    // Iterate over each argument in the configuration
    for index in 0..configuration.get_arguments().len() {
        // Clone the current argument
        let argument: String = configuration.get_arguments()[index].clone();

        // Attempt to parse the argument as a function
        let function: function::Function = match function::Function::from_str(&argument) {
            Ok(f) => f,
            Err(_) => continue, // If parsing fails, skip to the next argument
        };

        loop {
            info!(
                "Detected function, {}, when executing command: {}, executing the function...", 
                function.get_name(), 
                configuration
            );
            // Execute the function asynchronously and await the result
            let result: String = function.execute().await?;
            info!("Function, {}, executed successfully", function.get_name());

            println!("Function executed successfully with result: {}", result);
            println!("Do you want to proceed with this result? (yes/retry/abort)");

            let mut user_input = String::new();
            std::io::stdin().read_line(&mut user_input).expect("Failed to read input");
            let user_input = user_input.trim().to_lowercase();

            match user_input.as_str() {
                "yes" => {
                    // Proceed with the result
                    configuration.revise_argument(index, result);
                    break;
                }
                "retry" => {
                    // Retry the function execution
                    continue;
                }
                "abort" => {
                    error!("Execution aborted by the user");
                    std::process::exit(1);
                }
                _ => {
                    error!("Invalid input, execution aborted");
                    std::process::exit(1);
                }
            }
        }
    }
    // Return the result of the function execution
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Setup a logger
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // Parse command line arguments
    let arguments = Arguments::parse();

    // Check if the generate flag is set
    if arguments.generate {
        generate_template();
        return Ok(());
    }

    // Determine the configuration file to use
    let configurations_file: String;
    if let Some(configurations) = &arguments.configuration_files {
        let path = std::path::Path::new(configurations);
        configurations_file = determine_configuration_file(Some(path), &arguments);
    } else {
        configurations_file =
            determine_configuration_file(Some(std::path::Path::new(".")), &arguments);
    }

    // Load and parse the configuration file
    let configurations: Vec<Configuration> = serde_json::from_str(
        &std::fs::read_to_string(&configurations_file).expect("Failed to load configurations"),
    )
        .expect("Failed to parse configurations");

    // Iterate over each configuration and execute the commands
    for mut configuration in configurations {
        execute_argument_function(&mut configuration).await?;
        execute_command(&configuration);
    }
    
    Ok(())
}
