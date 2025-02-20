#[cfg(test)]
mod tests {
    use std::io::Write;
    use cchain::core::{chain::Chain, traits::Execution};
    use tempfile::NamedTempFile;

    // Test that Chain can be created from a valid JSON file
    #[test]
    fn test_chain_creation_from_file() {
        let programs = r#"[
            {
                "command": "echo",
                "arguments": ["hello"],
                "awaitable_variable": null,
                "remedy_command_line": null,
                "failure_handling_options": {
                    "exit_on_failure": true
                },
                "concurrency_group": null,
                "retry": 0
            }
        ]"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", programs).unwrap();

        let chain = Chain::from_file(temp_file.path().to_str().unwrap());
        assert!(chain.is_ok());
        let chain = chain.unwrap();
        assert!(!format!("{}", chain).is_empty());
    }

    // Test that syntax validation fails when using uninitialized variables
    #[test]
    fn test_validate_syntax_fails_with_uninitialized_variable() {
        let programs = r#"[
            {
                "command": "command-non-exist",
                "arguments": ["$<<hello>>"],
                "awaitable_variable": null,
                "remedy_command_line": null,
                "failure_handling_options": {
                    "exit_on_failure": true
                },
                "concurrency_group": null,
                "stdout_stored_to": null,
                "retry": 0
            },
            {
                "command": "echo",
                "arguments": ["$<<hello:on_program_execution>>"],
                "remedy_command_line": null,
                "failure_handling_options": {
                    "exit_on_failure": true
                },
                "concurrency_group": null,
                "retry": 0
            }
        ]"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", programs).unwrap();

        let mut chain = Chain::from_file(temp_file.path().to_str().unwrap()).unwrap();
        let result = chain.validate_syntax();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Check is not passed. 😢"
        );
    }

    // Test that failure count increments when a program execution fails
    #[test]
    fn test_program_failure_increments_counter() {
        let programs = r#"[
            {
                "command": "invalid_command_that_does_not_exist",
                "arguments": [],
                "awaitable_variable": null,
                "remedy_command_line": null,
                "failure_handling_options": {
                    "exit_on_failure": false
                },
                "concurrency_group": null,
                "retry": 0
            }
        ]"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", programs).unwrap();

        let mut chain = Chain::from_file(temp_file.path().to_str().unwrap()).unwrap();
        let result = chain.execute();

        assert!(result.is_ok());
        assert_eq!(chain.get_failed_program_execution_number(), 1);
    }
}