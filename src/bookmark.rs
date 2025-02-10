use anyhow::{Error, Result};
use dirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bookmark {
    configuration_paths: Vec<String>,
    bookmark_path: String,
}

impl Bookmark {
    pub fn from_file() -> Self {
        let bookmark_path = dirs::home_dir().unwrap().join(".cchain");

        if bookmark_path.exists() {
            let bookmark_file = std::fs::read_to_string(&bookmark_path).unwrap();
            serde_json::from_str(&bookmark_file).unwrap()
        } else {
            Bookmark {
                configuration_paths: Vec::new(),
                bookmark_path: bookmark_path.to_string_lossy().into_owned(),
            }
        }
    }

    pub fn save(&self) {
        let bookmark_file: String = serde_json::to_string(&self).unwrap();

        std::fs::write(&self.bookmark_path, bookmark_file).unwrap();
    }

    pub fn bookmark_configuration(&mut self, configuration_path: String) -> Result<(), Error> {
        if self.configuration_paths.contains(&configuration_path) {
            return Err(anyhow::anyhow!(
                "Configuration is likely duplicated: {}",
                configuration_path
            ));
        } else {
            self.configuration_paths.push(configuration_path);
            Ok(())
        }
    }

    pub fn unbookmark_configuration_by_index(&mut self, index: usize) -> Result<(), Error> {
        if index < self.configuration_paths.len() {
            self.configuration_paths.remove(index);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Index out of bounds: {}", index))
        }
    }

    pub fn unbookmark_configuration_by_path(
        &mut self,
        configuration_path: &str,
    ) -> Result<(), Error> {
        if let Some(pos) = self
            .configuration_paths
            .iter()
            .position(|x| x == configuration_path)
        {
            self.configuration_paths.remove(pos);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Configuration path not found: {}",
                configuration_path
            ))
        }
    }

    pub fn get_bookmarked_configurations(&self) -> &Vec<String> {
        &self.configuration_paths
    }

    pub fn get_bookmarked_configurations_by_index(&self, index: usize) -> Option<&String> {
        self.configuration_paths.get(index)
    }
}
