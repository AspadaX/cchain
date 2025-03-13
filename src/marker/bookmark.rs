use std::{path::{Path, PathBuf}, str::FromStr};

use anyhow::{anyhow, Error, Result};
use dirs;
use serde::{Deserialize, Serialize};

use crate::commons::naming::HumanReadable;

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
        let new_path: PathBuf = dirs::home_dir().unwrap().join(".cchain");

        if new_path.exists() {
            match std::fs::remove_file(&new_path) {
                Ok(_) => return Ok(()),
                Err(error) => {
                    return Err(anyhow!(
                        "Failed to delete existing bookmark file: {}",
                        error
                    ))
                }
            };
        }

        Ok(())
    }

    pub fn from_file() -> Self {
        let bookmark_path = dirs::home_dir().unwrap().join(".cchain");

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
        let chain_reference = ChainReference::from_str(&configuration_path)?;
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
        if let Some(pos) = self
            .chain_references
            .iter()
            .position(|x| x.get_chain_path_string() == configuration_path)
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

    /// Get all paths of the chains in the bookmark that are no longer exist
    /// in their original positions
    pub fn get_invalid_paths(&self) -> Result<Vec<String>, Error> {
        let mut invalid_paths: Vec<String> = Vec::new();
        for chain_reference in &self.chain_references {
            let path_stirng = chain_reference.get_chain_path_string();
            let path = Path::new(&path_stirng);

            // Save the non-exist paths to the invalid paths vec
            if !path.exists() {
                invalid_paths.push(path_stirng);
            }
        }

        Ok(invalid_paths)
    }

    pub fn get_chain_references(&self) -> &Vec<ChainReference> {
        &self.chain_references
    }

    pub fn get_chain_reference_by_index(&self, index: usize) -> Option<&ChainReference> {
        self.chain_references.get(index)
    }
    
    /// Search chains by using keywords
    pub fn get_chains_by_keywords(&self, keywords: Vec<String>) -> Option<Vec<&ChainReference>> {
        let keywords: Vec<String> = keywords.iter()
            .map(|keyword| keyword.to_lowercase())
            .collect::<Vec<String>>();
        let mut matched_chains: Vec<(&ChainReference, usize)> = Vec::new();
        
        // Iterate over the chain references
        for chain_reference in &self.chain_references {
            // Use human readable name to be searched
            let name: String = chain_reference.get_human_readable_name();
            let words: Vec<String> = name.split(" ")
                .map(|word| word.to_lowercase())
                .collect::<Vec<String>>();
            
            // Find the keyword in the name one by one
            for keyword in &keywords {
                // Skip if the keyword is empty
                if keyword.is_empty() {
                    continue;
                }
                
                // When a keyword is found in the name
                if words.contains(keyword) {
                    let mut is_existing: bool = false;
                    // Increment the match count if the chain is already in the list
                    for matched_chain in &mut matched_chains {
                        if matched_chain.0 == chain_reference {
                            matched_chain.1 += 1;
                            is_existing = true;
                        }
                    }
                    
                    // Add the chain to the list if the chain is not already in the list
                    if !is_existing {
                        matched_chains.push(
                            (chain_reference, 1)
                        );
                    }
                    
                    continue;
                }
            }
        }
        
        // Sort the chains by match count in descending order
        matched_chains
            .sort_by(|a, b| b.1.cmp(&a.1));
        
        let mut results = Vec::new();
        for matched_chain in matched_chains {
            // Skip the chains if the score is zero
            if matched_chain.1 != 0 {
                results.push(matched_chain.0);
            }
        }

        Some(results)
    }
}