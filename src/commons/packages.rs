use std::{collections::HashSet, env::consts::OS};
use std::path::PathBuf;
use anyhow::{anyhow, Error, Result};
use which::which;

use super::shell::execute_system_native_script;

/// Represents a package
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    name: String,
}

impl Package {
    pub fn new(name: String) -> Self {
        Package { name }
    }
    
    // Get a reference to the package name
    pub fn access_package_name(&self) -> &str {
        &self.name
    }
    
    pub fn get_available_packages() -> Result<HashSet<Package>, Error> {
        let output: String = if cfg!(target_os = "windows") {
            // Windows system: use 'where' command to list available commands
            execute_system_native_script("where /Q *")?
        } else {
            // Unix system: use 'compgen -c' to list available commands
            execute_system_native_script("compgen -c")?
        };
    
        Ok(
            output 
                .lines()
                .map(|s| Package { name: s.to_string() })
                .collect()
        )
    }
}

/// Represents a package manager
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageManager {
    name: String,
    path: PathBuf,
}

impl PackageManager {
    pub fn install_package(&self, package_name: &str) -> Result<(), Error> {
        let command = match self.name.as_str() {
            "brew" => format!("brew install {}", package_name),
            "MacPorts" => format!("port install {}", package_name),
            "apt" => format!("sudo apt-get install -y {}", package_name),
            "snap" => format!("sudo snap install {}", package_name),
            "yum" => format!("sudo yum install -y {}", package_name),
            "dnf" => format!("sudo dnf install -y {}", package_name),
            "pacman" => format!("sudo pacman -S --noconfirm {}", package_name),
            "zypper" => format!("sudo zypper install -y {}", package_name),
            "emerge" => format!("sudo emerge {}", package_name),
            "choco" => format!("choco install -y {}", package_name),
            "scoop" => format!("scoop install {}", package_name),
            "winget" => format!("winget install --id {}", package_name),
            _ => return Err(anyhow!("Unsupported package manager: {}", self.name)),
        };
        
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .status()?;
        
        if !status.success() {
            return Err(anyhow!("Failed to install package: {}, error code: {}", package_name, status.code().unwrap()));
        } 
        
        Ok(())
    }
    
    fn check_package_managers_availability<'a, T: Iterator<Item = &'a str>>(package_managers: T) -> Vec<PackageManager> {
        let mut results: Vec<PackageManager> = Vec::new();
        for package_manager in package_managers {
            if let Ok(path_buf) = which(package_manager) {
                results.push(
                    PackageManager {
                        name: package_manager.to_string(),
                        path: path_buf,
                    }
                );
            }
        }
        
        results
    }
    
    pub fn get_available_package_managers() -> Result<Vec<PackageManager>, Error> {
        // Get the current operating system
        let os: &str = OS;
        let mut package_managers: Vec<PackageManager> = Vec::new();
        
        match os {
            "macos" => {
                let macos_package_managers: [&str; 2] = ["brew", "MacPorts"];
                package_managers.extend(PackageManager::check_package_managers_availability(macos_package_managers.into_iter()));
            }
            "linux" => {
                // List of common Linux package managers (name, command)
                let pm_commands: [&str; 7] = ["apt", "snap", "yum", "dnf", "pacman", "zypper", "emerge"];
                package_managers.extend(PackageManager::check_package_managers_availability(pm_commands.into_iter()));
            }
            "windows" => {
                // Windows package managers
                let pm_commands: [&str; 3] = ["choco", "scoop", "winget"];
                package_managers.extend(PackageManager::check_package_managers_availability(pm_commands.into_iter()));
            }
            _ => {
                return Err(anyhow!("Unsupported OS platform: {}. Please install a package manager, or file an issue on GitHub.", os));
            }
        }
    
        if package_managers.is_empty() {
            return Err(anyhow!("No package managers detected."));
        }
        
        Ok(package_managers)
    }
}

/// Impl this trait to verify if the package manager is available.
pub trait AvailablePackageManager {
    fn get_available_package_managers(&self) -> bool;
}

/// Impl this trait to verify if the packages are available
pub trait AvailablePackages {
    /// Get required packages
    fn get_required_packages(&self) -> Result<HashSet<Package>, Error>;
    
    /// Return missed packages
    fn get_missing_packages(&self) -> Result<HashSet<Package>, Error> {
        let required_packages: HashSet<Package> = self.get_required_packages()?;
        let available_packages: HashSet<Package> = Package::get_available_packages()?;
        
        Ok(
            required_packages.into_iter()
                .filter(|pkg| !available_packages.contains(pkg))
                .collect()
        )
    }
}