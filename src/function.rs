use std::str::FromStr;

use async_openai::{
    config::OpenAIConfig, 
    types::{
        ChatCompletionRequestMessageContentPartTextArgs, 
        ChatCompletionRequestUserMessageArgs, 
        CreateChatCompletionRequestArgs, 
        CreateChatCompletionResponse
    }, 
    Client
};
use regex;

pub struct Function {
    name: String,
    parameters: Vec<String>,
}

impl FromStr for Function {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r"(\w+)\s*\(\s*'(.*)'\s*,\s*'(.*)'\s*\)")?;

        if let Some(caps) = re.captures(s) {
            let func_name: String = caps.get(1).ok_or_else(|| anyhow::anyhow!("Failed to capture function name"))?.as_str().to_string();
            let arg1 = caps.get(2).ok_or_else(|| anyhow::anyhow!("Failed to capture first argument"))?.as_str().to_string();
            let arg2 = caps.get(3).ok_or_else(|| anyhow::anyhow!("Failed to capture second argument"))?.as_str().to_string();

            return Ok(Function {
                name: func_name,
                parameters: vec![arg1, arg2],
            });
        }

        Err(anyhow::anyhow!("No function found"))
    }
}

impl Function {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_parameters(&self) -> &Vec<String> {
        &self.parameters
    }

    pub async fn execute(&self) -> Result<String, anyhow::Error> {
        match self.name.as_str() {
            "llm_generate" => self.llm_generate().await,
            _ => Err(anyhow::anyhow!("Function not found")),
        }
    }

    async fn llm_generate(&self) -> Result<String, anyhow::Error> {
        let api_base: String = std::env::var("CCHAIN_OPENAI_API_BASE")?;
        let api_key: String = std::env::var("CCHAIN_OPENAI_API_KEY")?;
        let model: String = std::env::var("CCHAIN_OPENAI_MODEL")?;
        
        let llm_configuration: OpenAIConfig = OpenAIConfig::default()
            .with_api_key(api_key)
            .with_api_base(api_base);
        let client: Client<OpenAIConfig> = async_openai::Client::with_config(
            llm_configuration
        );
        
        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(vec![ChatCompletionRequestUserMessageArgs::default()
                .content(vec![
                    ChatCompletionRequestMessageContentPartTextArgs::default()
                        .text(
                            format!(
                                "{}\n{}\n", 
                                self.parameters[0], self.parameters[1]
                            )
                        )
                        .build()?
                        .into()
                ])
                .build()?
                .into()])
            .build()?;

        let response: CreateChatCompletionResponse = client
            .chat()
            .create(request)
            .await?;
        
        Ok(response.choices[0].clone().message.content.unwrap())
    }
}
