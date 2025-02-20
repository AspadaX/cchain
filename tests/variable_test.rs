#[cfg(test)]
mod tests {
    use cchain::variable::{Variable, VariableInitializationTime, VariableLifetime};


    #[test]
    fn test_extract_no_variables() {
        let input = "No variables here";
        assert!(Variable::extract_variable_names(input).is_empty());
    }

    #[test]
    fn test_extract_single_variable() {
        let input = "<<test>>";
        assert_eq!(Variable::extract_variable_names(input), vec!["test"]);
    }

    #[test]
    fn test_extract_multiple_variables() {
        let input = "<<var1>> and <<var2:on_program_execution>>";
        let vars = Variable::extract_variable_names(input);
        assert_eq!(vars, vec!["var1", "var2:on_program_execution"]);
    }

    #[test]
    fn test_extract_adjacent_variables() {
        let input = "<<var1>><<var2>>";
        let vars = Variable::extract_variable_names(input);
        assert_eq!(vars, vec!["var1", "var2"]);
    }

    #[test]
    fn test_parse_variables_with_different_qualifiers() {
        let input = r#"
            <<on_chain_var>> 
            <<exec_var:on_program_execution>>
            <<await_var>> 
        "#;

        let vars = Variable::parse_variables_from_str(input, 0).unwrap();

        // Test OnChainStartup variable
        assert_eq!(vars[0].get_variable_name(), "on_chain_var");
        assert!(matches!(
            vars[0].get_initialization_time(),
            VariableInitializationTime::OnChainStartup(_)
        ));

        // Test OnProgramExecution variable
        assert_eq!(vars[1].get_variable_name(), "exec_var");
        assert!(matches!(
            vars[1].get_initialization_time(),
            VariableInitializationTime::OnProgramExecution(_)
        ));

        // Test Await variable (requires separate parsing via parse_await_variable)
        let await_var = Variable::parse_await_variable("<<await_var>>", 0);
        assert_eq!(await_var.get_variable_name(), "await_var");
        assert_eq!(await_var.get_raw_variable_name(), "<<await_var>>");
        assert!(matches!(
            await_var.get_initialization_time(),
            VariableInitializationTime::Await(_)
        ));
    }

    #[test]
    fn test_case_insensitive_qualifier() {
        let input = "<<var:ON_PROGRAM_EXECUTION>>";
        let vars = Variable::parse_variables_from_str(input, 0).unwrap();

        assert!(matches!(
            vars[0].get_initialization_time(),
            VariableInitializationTime::OnProgramExecution(_)
        ));
    }

    #[test]
    fn test_invalid_qualifier_fallback() {
        let input = "<<var:invalid_qualifier>>";
        let vars = Variable::parse_variables_from_str(input, 0).unwrap();

        assert!(matches!(
            vars[0].get_initialization_time(),
            VariableInitializationTime::OnChainStartup(_)
        ));
    }

    #[test]
    fn test_variable_name_parsing() {
        let input = "<<namespace::var_name:on_program_execution>>";
        let vars = Variable::parse_variables_from_str(input, 0).unwrap();

        assert_eq!(vars[0].get_raw_variable_name(), input);
        assert_eq!(vars[0].get_variable_name(), "namespace::var_name");
        assert!(matches!(
            vars[0].get_initialization_time(),
            VariableInitializationTime::OnProgramExecution(_)
        ));
    }

    #[test]
    fn test_mixed_variable_syntax() {
        let input = r#"
            User <<$name>> needs <<count:on_program_execution>> items.
            Final result: <<result>>.
        "#;

        let vars = Variable::parse_variables_from_str(input, 0).unwrap();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0].get_variable_name(), "$name");
        assert_eq!(vars[1].get_variable_name(), "count");
        assert_eq!(vars[2].get_variable_name(), "result");
    }

    #[test]
    fn test_parse_variables_from_str() {
        let input = "<<var1>> <<var2:on_program_execution>>";
        let vars = Variable::parse_variables_from_str(input, 3).unwrap();
        assert_eq!(vars.len(), 2);

        assert_eq!(vars[0].get_variable_name(), "var1");
        assert!(matches!(
            vars[0].get_initialization_time(),
            VariableInitializationTime::OnChainStartup(_)
        ));

        assert_eq!(vars[1].get_variable_name(), "var2");
        assert!(matches!(
            vars[1].get_initialization_time(),
            VariableInitializationTime::OnProgramExecution(_)
        ));
    }

    #[test]
    fn test_parse_await_variable() {
        let var = Variable::parse_await_variable("<<await_var>>", 2);
        assert_eq!(var.get_variable_name(), "await_var");
        assert!(matches!(
            var.get_initialization_time(),
            VariableInitializationTime::Await(_)
        ));
    }

    #[test]
    fn test_human_readable_name() {
        let var = Variable::new(
            "my_variable".to_string(),
            None,
            None,
            VariableInitializationTime::OnChainStartup(VariableLifetime::new(None)),
        );
        assert_eq!(var.get_human_readable_name(), "My Variable");

        let var_custom = Variable::new(
            "var".to_string(),
            None,
            Some("Custom Name".to_string()),
            VariableInitializationTime::OnChainStartup(VariableLifetime::new(None)),
        );
        assert_eq!(var_custom.get_human_readable_name(), "Custom Name");
    }

    #[test]
    fn test_register_and_get_value() {
        let mut var = Variable::new(
            "test".to_string(),
            None,
            None,
            VariableInitializationTime::OnChainStartup(VariableLifetime::new(None)),
        );
        var.register_value("value");
        assert_eq!(var.get_value().unwrap(), "value");
    }

    #[test]
    fn test_get_uninitialized_value() {
        let var = Variable::new(
            "test".to_string(),
            None,
            None,
            VariableInitializationTime::OnChainStartup(VariableLifetime::new(None)),
        );
        assert!(var.get_value().is_err());
    }

    #[test]
    fn test_raw_variable_name() {
        let var_chain = Variable::new(
            "var".to_string(),
            None,
            None,
            VariableInitializationTime::OnChainStartup(VariableLifetime::new(None)),
        );
        assert_eq!(var_chain.get_raw_variable_name(), "<<var>>");

        let var_program_exec = Variable::new(
            "var".to_string(),
            None,
            None,
            VariableInitializationTime::OnProgramExecution(VariableLifetime::new(Some(1))),
        );
        assert_eq!(
            var_program_exec.get_raw_variable_name(),
            "<<var:on_program_execution>>"
        );
    }

    #[test]
    fn test_is_initialized() {
        let init_on_chain = VariableInitializationTime::OnChainStartup(VariableLifetime::new(None));
        assert!(init_on_chain.is_initialized(0));
        assert!(!init_on_chain.is_initialized(1));

        let init_on_program = VariableInitializationTime::OnProgramExecution(VariableLifetime::new(Some(2)));
        assert!(init_on_program.is_initialized(2));
        assert!(!init_on_program.is_initialized(1));

        let init_await = VariableInitializationTime::Await(VariableLifetime::new(Some(3)));
        assert!(init_await.is_initialized(3));
        assert!(!init_await.is_initialized(2));
    }
}