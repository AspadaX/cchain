use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Error, Result};
use dirs;
use serde::{Deserialize, Serialize};

use super::reference::ChainReference;

/// `Bookmark` is a collection of references to the chains
/// `ChainRefenence` is a reference to a chain
#[derive(Debug, Serialize, Deserialize)]
pub struct Bookmark {
    chain_references: Vec<ChainReference>,
    bookmark_path: String,
}

impl Bookmark {
    pub fn reset() -> Result<(), Error> {
        let new_path: PathBuf = dirs::home_dir()
            .unwrap()
            .join(".cchain");
        
        if new_path.exists() {
            match std::fs::remove_file(&new_path) {
                Ok(_) => return Ok(()),
                Err(error) => return Err(
                    anyhow!("Failed to delete existing bookmark file: {}", error)
                )
            };
        }
        
        Ok(())
    }
    
    pub fn from_file() -> Self {
        let bookmark_path = dirs::home_dir()
            .unwrap()
            .join(".cchain");

        if bookmark_path.exists() {
            let bookmark_file = std::fs::read_to_string(&bookmark_path).unwrap();
            serde_json::from_str(&bookmark_file).unwrap()
        } else {
            Bookmark {
                chain_references: Vec::new(),
                bookmark_path: bookmark_path.to_string_lossy().into_owned(),
            }
        }
    }

    pub fn save(&self) {
        let bookmark_file: String = serde_json::to_string(&self).unwrap();

        std::fs::write(&self.bookmark_path, bookmark_file).unwrap();
    }

    pub fn add_chain_reference(&mut self, configuration_path: String) -> Result<(), Error> {
        let chain_reference = ChainReference::from_str(
            &configuration_path
        )?;
        if self.chain_references.contains(&chain_reference) {
            return Err(anyhow::anyhow!(
                "Configuration is likely duplicated: {}",
                &configuration_path
            ));
        } else {
            self.chain_references.push(chain_reference);
            Ok(())
        }
    }

    pub fn remove_chain_reference_by_index(&mut self, index: usize) -> Result<(), Error> {
        if index < self.chain_references.len() {
            self.chain_references.remove(index);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Index out of bounds: {}", index))
        }
    }

    pub fn remove_chain_reference_by_path(
        &mut self,
        configuration_path: &str,
    ) -> Result<(), Error> {
        let chain_reference = ChainReference::from_str(
            configuration_path
        )?;
        if let Some(pos) = self
            .chain_references
            .iter()
            .position(|x| x == &chain_reference)
        {
            self.chain_references.remove(pos);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Configuration path not found: {}",
                configuration_path
            ))
        }
    }

    pub fn get_chain_references(&self) -> &Vec<ChainReference> {
        &self.chain_references
    }

    pub fn get_chain_reference_by_index(&self, index: usize) -> Option<&ChainReference> {
        self.chain_references.get(index)
    }
}