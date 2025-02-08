use std::str::FromStr;

use anyhow::Error;
use regex;

pub struct Variable {
    /// The name for the variable
    name: String,
    /// The actual value for the variable
    value: Option<String>,
    /// The name for users to read on the screen
    human_readable_name: String,
    /// The message for prompting user input if needed
    request_message: String
}

impl FromStr for Variable {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r"(\w+)\s*\(\s*'(.*)'\s*,\s*'(.*)'\s*\)")?;

        if let Some(caps) = re.captures(s) {
            let func_name: String = caps
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Failed to capture function name"))?
                .as_str()
                .to_string();
            let arg1 = caps
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Failed to capture first argument"))?
                .as_str()
                .to_string();
            let arg2 = caps
                .get(3)
                .ok_or_else(|| anyhow::anyhow!("Failed to capture second argument"))?
                .as_str()
                .to_string();

            return Ok(Function {
                name: func_name,
                parameters: vec![arg1, arg2],
            });
        }

    }
}