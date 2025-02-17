use serde::{Deserialize, Serialize};

/// Currently supported interpreters
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, PartialOrd)]
pub enum Interpreter {
    #[serde(alias = "sh")]
    Sh,
}
