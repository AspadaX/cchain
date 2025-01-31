use clap::Parser;
use serde::{Deserialize, Serialize};
use log::{info, error};

#[derive(Parser)]
pub struct Arguments {
    // path to the command line chain configuration file
    #[clap(short, long, default_value = "./configurations.json")]
    pub configurations: String,
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
    
    let arguments = Arguments::parse();

    let configurations: Vec<Configuration> = serde_json::from_str(
        &std::fs::read_to_string(&arguments.configurations)
            .expect("Failed to load configurations")
    )
        .expect("Failed to parse configurations");

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
        while !status.success() && (configuration.retry == 0 || attempts < configuration.retry) {
            attempts += 1;
            info!("Retrying command: {:?}, attempt: {}", configuration.command, attempts);
            // Spawn the command again as a child process
            let status = command.spawn().expect("Failed to execute command").wait().expect("Failed to wait on child");
            // If the command fails again and retry limit is reached, print an error message
            if !status.success() && configuration.retry != 0 && attempts >= configuration.retry {
                error!("Failed to execute command: {:?}", configuration.command);
            }
        }
    }
}
