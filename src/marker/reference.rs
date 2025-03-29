use std::{collections::HashSet, path::Path, str::FromStr};

use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};

use crate::{commons::{naming::HumanReadable, packages::{AvailablePackages, Package}}, core::chain::Chain};

/// Provide methods to track the path of a data structure
pub trait TrackPath {
    fn get_path(&self) -> &str;
}

/// `Bookmark` is a collection of references to the chains
/// `ChainRefenence` is a reference to a chain
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainReference {
    /// Path to the chain
    chain_path: String,
}

impl ChainReference {
    pub fn new(path: String) -> Self {
        Self { chain_path: path }
    }

    /// Return a canonicalized path of the chain
    pub fn get_chain_path_string(&self) -> String {
        self.chain_path.clone()
    }
}

impl FromStr for ChainReference {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let path = Path::new(s);
        if path.exists() {
            if path.is_file()
                && path.extension().map_or(false, |ext| ext == "json")
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("cchain_")
            {
                return Ok(Self {
                    chain_path: s.to_string(),
                });
            } else {
                return Err(anyhow!("Chain at {} has a wrong naming convention", s));
            }
        } else {
            return Err(anyhow!("Chain at {} does not exist", s));
        }
    }
}

impl HumanReadable for ChainReference {
    fn get_raw_name(&self) -> String {
        let path = Path::new(&self.chain_path);

        path.file_name().unwrap().to_string_lossy().to_string()
    }

    fn get_human_readable_name(&self) -> String {
        let result: String = self
            .get_raw_name()
            .split('_')
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        result
            .trim_start_matches("Cchain")
            .to_string()
            .trim_end_matches(".json")
            .to_string()
    }
}

impl AvailablePackages for ChainReference {
    fn get_required_packages(&self) -> Result<HashSet<Package>, Error> {
        let chain = Chain::from_file(&self.chain_path)?;
        chain.get_missing_packages()
    }
}

impl TrackPath for ChainReference {
    fn get_path(&self) -> &str {
        &self.chain_path
    }
}
