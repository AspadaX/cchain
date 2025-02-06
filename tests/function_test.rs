#[cfg(test)]
mod tests {
    use cchain::function::Function;
    use std::str::FromStr;

    #[test]
    fn test_from_str() {
        let func_str = r#"llm_generate('param1', 'param2')"#;
        let function = Function::from_str(func_str).unwrap();

        assert_eq!(function.get_name(), "llm_generate");
        assert_eq!(
            function.get_parameters(),
            &vec!["param1".to_string(), "param2".to_string()]
        );
    }

    #[test]
    fn test_from_str_invalid() {
        let func_str = "invalid_function_string";
        let function = Function::from_str(func_str);

        assert!(function.is_err());
    }

    #[test]
    fn test_get_name() {
        let function = Function::from_str("test_function('param1', 'param2')").unwrap();

        assert_eq!(function.get_name(), "test_function");
    }

    #[test]
    fn test_get_parameters() {
        let function = Function::from_str("test_function('param1', 'param2')").unwrap();

        assert_eq!(
            function.get_parameters(),
            &vec!["param1".to_string(), "param2".to_string()]
        );
    }
}
