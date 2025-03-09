#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::collections::HashMap;
    use cchain::core::{command::CommandLine, interpreter::Interpreter, traits::Execution};

    #[test]
    #[cfg(unix)]
    fn test_execute_sh_command() -> Result<()> {
        let mut cmd = CommandLine::new(
            "printf".to_string(),
            vec!["%s".to_string(), "test".to_string()],
            Some(Interpreter::Sh),
            None,
            None,
        );
        let results = cmd.execute()?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_output(), "test");
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_environment_variable_override() -> Result<()> {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "success".to_string());
        let mut cmd = CommandLine::new(
            "sh".to_string(),
            vec!["-c".to_string(), "echo $TEST_VAR".to_string()],
            None,
            Some(env_vars),
            None,
        );
        let results = cmd.execute()?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_output().trim(), "success");
        Ok(())
    }

    #[test]
    fn test_inject_value_into_arguments() -> Result<()> {
        let mut cmd = CommandLine::new(
            "echo".to_string(),
            vec!["Hello, <<NAME>>!".to_string()],
            None,
            None,
            None,
        );
        cmd.inject_value_to_variables("<<NAME>>", "Alice".to_string())?;
        assert_eq!(*cmd.get_arguments(), vec!["Hello, Alice!"]);
        Ok(())
    }

    #[test]
    fn test_revise_argument_by_index() {
        let mut cmd = CommandLine::new(
            "echo".to_string(),
            vec!["old".to_string()],
            None,
            None,
            None,
        );
        cmd.revise_argument_by_index(0, "new".to_string());
        assert_eq!(*cmd.get_arguments(), vec!["new".to_string()]);
    }

    #[test]
    #[cfg(unix)]
    fn test_execute_invalid_command() {
        let mut cmd = CommandLine::new(
            "nonexistentcommand".to_string(),
            vec!["".to_string()],
            None,
            None,
            None,
        );
        let result = cmd.execute();
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.starts_with("Failed to execute Command Line"));
    }
}