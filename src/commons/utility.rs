use std::fs::DirEntry;
use std::io::Write;

use anyhow::{Error, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::ChatCompletionRequestMessageContentPartTextArgs;
use async_openai::types::ChatCompletionRequestUserMessageArgs;
use async_openai::types::CreateChatCompletionRequestArgs;
use async_openai::types::CreateChatCompletionResponse;
use async_openai::Client;

use crate::display_control::display_message;
use crate::display_control::Level;

pub fn get_paths(path: &std::path::Path) -> Result<Vec<DirEntry>, Error> {
    let mut paths: Vec<DirEntry> = Vec::new();
    let entries = std::fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        if entry.path().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "json")
            && entry.file_name().to_string_lossy().starts_with("cchain_")
        {
            paths.push(entry);
        }
    }
    Ok(paths)
}

pub fn input_message(prompt: &str) -> Result<String, Error> {
    // display the prompt message for inputting values
    display_message(Level::Input, prompt);
    // collect the input as a string
    let mut input = String::new();
    // receive stdin
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;

    Ok(input)
}

/// Generate a text with a given prompt. This function automatically resolves 
/// the environment variables needed
pub fn generate_text_with_llm(prompt: String) -> Result<String, Error> {
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(
        async {
            let api_base: String = std::env::var("CCHAIN_OPENAI_API_BASE")?;
            let api_key: String = std::env::var("CCHAIN_OPENAI_API_KEY")?;
            let model: String = std::env::var("CCHAIN_OPENAI_MODEL")?;

            let llm_configuration: OpenAIConfig = OpenAIConfig::default()
                .with_api_key(api_key)
                .with_api_base(api_base);
            let client: Client<OpenAIConfig> = async_openai::Client::with_config(llm_configuration);

            let request = CreateChatCompletionRequestArgs::default()
                .model(model)
                .messages(vec![ChatCompletionRequestUserMessageArgs::default()
                    .content(vec![
                        ChatCompletionRequestMessageContentPartTextArgs::default()
                            .text(prompt)
                            .build()?
                            .into(),
                    ])
                    .build()?
                    .into()])
                .build()?;

            let response: CreateChatCompletionResponse =
                match client.chat().create(request.clone()).await {
                    std::result::Result::Ok(response) => response,
                    Err(e) => {
                        anyhow::bail!("Failed to execute function: {}", e);
                    }
                };
            
            return Ok(response.choices[0].clone().message.content.unwrap());
        }
    )?;

    Ok(result)
}