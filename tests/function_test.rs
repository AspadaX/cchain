#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use cchain::function::Function;

    #[test]
    fn test_from_str() {
        let func_str = r#"llm_generate('param1', 'param2')"#;
        let function = Function::from_str(func_str).unwrap();

        assert_eq!(function.get_name(), "llm_generate");
        assert_eq!(function.get_parameters(), &vec!["param1".to_string(), "param2".to_string()]);
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

        assert_eq!(function.get_parameters(), &vec!["param1".to_string(), "param2".to_string()]);
    }

    #[tokio::test]
    async fn test_execute_llm_generate() {
        let function = Function::from_str("llm_generate('param1', 'param2')").unwrap();

        // Mocking environment variables
        std::env::set_var("CCHAIN_OPENAI_API_BASE", "http://192.168.0.101:11434/v1");
        std::env::set_var("CCHAIN_OPENAI_API_KEY", "test_api_key");
        std::env::set_var("CCHAIN_OPENAI_MODEL", "mistral");

        // Mocking the OpenAI client
        // Note: Mocking the OpenAI client is complex and requires a proper mocking framework.
        // Here we assume the existence of a mock client for demonstration purposes.
        // You need to implement the mock client and its behavior as per your requirements.

        let result = function.execute().await;
        assert!(result.is_ok());
    }
}
