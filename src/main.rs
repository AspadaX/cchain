use clap::Parser;
use serde::{Deserialize, Serialize};
use log::{info, error};

#[derive(Parser)]
pub struct Arguments {
    // path to the command line chain configuration file
    #[clap(short, long)]
    pub configurations: Option<String>,
    
    // generate a configuration template
    #[clap(short, long)]
    pub generate: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct Configuration {
    command: String,
    arguments: Vec<String>,
    retry: i32,
}

fn main() {
    // Setup a logger
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // Parse command line arguments
    let arguments = Arguments::parse();

    // Check if the generate flag is set
    if arguments.generate.unwrap_or(false) {
        // Create a template configuration
        let template = Configuration {
            command: "example_command".to_string(),
            arguments: vec!["arg1".to_string(), "arg2".to_string()],
            retry: 3,
        };
        // Serialize the template to JSON
        let template_json = serde_json::to_string_pretty(&template).expect("Failed to serialize template");
        // Write the template JSON to a file
        std::fs::write("cchain_template.json", template_json).expect("Failed to write template file");
        info!("Template configuration file generated: cchain_template.json");
        return;
    }

    // Determine the configuration file to use
    let configurations_file = if let Some(config_file) = arguments.configurations {
        config_file
    } else {
        // Read the current directory for configuration files
        let paths = std::fs::read_dir(".").expect("Failed to read current directory")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file() && entry.file_name().to_string_lossy().starts_with("cchain_") && entry.path().extension().map_or(false, |ext| ext == "json"))
            .collect::<Vec<_>>();

        // If no configuration files are found, log an error and return
        if paths.is_empty() {
            error!("No configuration files found starting with 'cchain_'");
            return;
        }

        // List available configuration files for the user to select
        println!("\nAvailable configuration files:");
        for (i, path) in paths.iter().enumerate() {
            println!("{}: {}", i, path.file_name().to_string_lossy());
        }

        // Prompt the user to select a configuration file
        println!("\nPlease select a configuration file to execute by entering the corresponding number:");
        let mut selection = String::new();
        std::io::stdin().read_line(&mut selection).expect("Failed to read input");
        let index: usize = selection.trim().parse().expect("Invalid selection");

        // Return the selected configuration file path
        paths[index].path().to_string_lossy().to_string()
    };

    // Load and parse the configuration file
    let configurations: Vec<Configuration> = serde_json::from_str(
        &std::fs::read_to_string(&configurations_file)
            .expect("Failed to load configurations")
    )
        .expect("Failed to parse configurations");

    // Iterate over each configuration and execute the commands
    for configuration in configurations {
        // Create a new command based on the configuration
        let mut command = std::process::Command::new(&configuration.command);
        // Add the arguments to the command
        command.args(&configuration.arguments);

        // Spawn the command as a child process
        let mut child = command.spawn().expect("Failed to execute command");
        // Wait for the child process to finish
        let status = child.wait().expect("Failed to wait on child");

        // If the command failed and retry is enabled, try to execute it again
        let mut attempts = 0;
        while !status.success() && (configuration.retry == -1 || attempts < configuration.retry) {
            attempts += 1;
            info!("\nRetrying command: {:?}, attempt: {}", configuration.command, attempts);
            // Spawn the command again as a child process
            let status = command.spawn().expect("Failed to execute command").wait().expect("Failed to wait on child");
            // If the command fails again and retry limit is reached, print an error message and stop the chain
            if !status.success() && configuration.retry != -1 && attempts >= configuration.retry {
                error!("\nFailed to execute command: {:?}", configuration.command);
                return;
            }
        }

        // If the command fails and retry is -1, keep retrying indefinitely
        if !status.success() && configuration.retry == -1 {
            loop {
                attempts += 1;
                info!("\nRetrying command: {:?}, attempt: {}", configuration.command, attempts);
                let status = command.spawn().expect("Failed to execute command").wait().expect("Failed to wait on child");
                if status.success() {
                    break;
                }
            }
        }

        // If the command fails and retry is 0, stop the chain
        if !status.success() && configuration.retry == 0 {
            error!("\nFailed to execute command: {:?}", configuration.command);
            return;
        }
    }
}
