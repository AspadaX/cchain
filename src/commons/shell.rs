use std::process::{Command, Output};

use anyhow::{anyhow, Error};

/// Provide translations into system native script
/// This is an attempt to shift from using command line execution to using system-native scripts,
/// such as Bash on Unix or PowerShell on Windows.
pub trait SystemScript {
    /// Get the system native script
    fn get_shell_script(&self) -> String;
    
    /// Execute the system native script
    fn execute(&self) -> Result<(), Error> {
        let script: String = self.get_shell_script();
        
        #[cfg(unix)]
        {
            let output: std::process::Output = Command::new("bash")
                .arg("-c")
                .arg(&script)
                .output()?;
            
            if output.status.success() {
                Ok(())
            } else {
                Err(anyhow!("Shell script execution failed with status: {}", output.status.code().unwrap_or(-1)))
            }
        }

        #[cfg(windows)]
        {
            let output: std::process::Output = Command::new("powershell")
                .arg("-Command")
                .arg(&script)
                .output()?;
            
            if output.status.success() {
                Ok(())
            } else {
                Err(anyhow!("Shell script execution failed with status: {}", output.status.code().unwrap_or(-1)))
            }
        }
    }
}

fn retrieve_script_output(output: Output) -> Result<String, Error> {
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(anyhow!(
            "Shell script execution failed with status: {}. Stderr: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Execute a system native script using the appropriate shell.
/// Return the stdout as a String
pub fn execute_system_native_script(script: &str) -> Result<String, Error> {
    #[cfg(unix)]
    let output: Output = {
        Command::new("bash")
            .arg("-c")
            .arg(script)
            .output()?
    };

    #[cfg(windows)]
    let output: Output = {
        Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .output()?
    };
    
    Ok(retrieve_script_output(output)?)
}
