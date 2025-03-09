#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;
    use cchain::core::{command::CommandLine, interpreter::Interpreter, options::{FailureHandlingOptions, StdoutStorageOptions}, program::Program, traits::Execution};

    #[test]
    fn test_execute_success() -> Result<()> {
        let mut program = Program::new(
            "echo".to_string(),
            vec!["test".to_string()],
            None,
            None,
            None,
            StdoutStorageOptions::default(),
            None,
            FailureHandlingOptions::default(),
            None,
            0,
        );
        let result = program.execute()?;
        assert!(!result[0].clone().get_output().is_empty());
        Ok(())
    }

    #[test]
    fn test_retry_failure() {
        let mut program = Program::new(
            "false".to_string(),
            vec![],
            None,
            None,
            None,
            StdoutStorageOptions::default(),
            None,
            FailureHandlingOptions::default(),
            None,
            2,
        );
        let result = program.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_stdout_storage_options() -> Result<()> {
        let mut program = Program::new(
            "printf".to_string(),
            vec!["test\n".to_string()],
            None,
            None,
            None,
            StdoutStorageOptions {
                without_newline_characters: true,
            },
            None,
            FailureHandlingOptions::default(),
            None,
            0,
        );
        let result = program.execute()?;
        assert_eq!(result[0].clone().get_output(), "test");
        Ok(())
    }

    #[test]
    fn test_execute_remedy_command_line() -> Result<()> {
        let mut program = Program::new(
            "echo".to_string(),
            vec!["test".to_string()],
            None,
            None,
            None,
            StdoutStorageOptions::default(),
            None,
            FailureHandlingOptions {
                exit_on_failure: true,
                remedy_command_line: Some(
                    CommandLine::new(
                        "echo".to_string(), 
                        vec!["hello".to_string()], 
                        Some(Interpreter::Sh), 
                        None,
                        None
                    )
                )
            },
            None,
            0,
        );
        program.execute_remedy_command_line()?;
        Ok(())
    }

    #[test]
    fn test_program_from_str() {
        let mut program = Program::from_str("echo hello world").unwrap();
        assert_eq!(program.get_command_line().get_command(), "echo");
        assert_eq!(
            program.get_command_line().get_arguments(),
            &vec!["hello".to_string(), "world".to_string()]
        );
    }

    #[test]
    fn test_get_concurrency_group() {
        let program = Program::new(
            "echo".to_string(),
            vec!["${{env::TEST_ARG_FUNCTION}}".to_string()],
            None,
            None,
            None,
            StdoutStorageOptions::default(),
            None,
            FailureHandlingOptions::default(),
            Some(3),
            0,
        );
        assert_eq!(program.get_concurrency_group(), Some(3));
    }

    #[test]
    fn test_default_program() {
        let program = Program::default();
        assert_eq!(*program.get_retry(), 0);
        assert!(program.get_awaitable_variable().is_none());
        assert_eq!(program.get_concurrency_group(), None);
    }
}