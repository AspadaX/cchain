use std::str::FromStr;

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
    },
    Client,
};
use regex;

use crate::display_control::{display_message, Level};

pub struct Function {
    name: String,
    parameters: Vec<String>,
}

impl FromStr for Function {
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

    pub fn execute(&self) -> Result<String, anyhow::Error> {
        match self.name.as_str() {
            "llm_generate" => self.llm_generate(),
            _ => Err(anyhow::anyhow!("Function not found")),
        }
    }

    fn llm_generate(&self) -> Result<String, anyhow::Error> {
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

                // execute the second parameter in the terminal and then get the output
                let command_output: String = if self.parameters.len() > 1 {
                    let parts: Vec<&str> = self.parameters[1].split_whitespace().collect();
                    let output = tokio::process::Command::new(parts[0])
                        .args(&parts[1..])
                        .output()
                        .await
                        .expect("Failed to execute command");

                    if !output.status.success() {
                        // Check if the command failed
                        let error_message = if !output.stderr.is_empty() {
                            String::from_utf8_lossy(&output.stderr).to_string()
                        } else {
                            format!("Command exited with status: {}", output.status)
                        };
                        return Err(anyhow::anyhow!("Command failed: {}", error_message));
                    }

                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    String::new()
                };

                let request = CreateChatCompletionRequestArgs::default()
                    .model(model)
                    .messages(vec![ChatCompletionRequestUserMessageArgs::default()
                        .content(vec![
                            ChatCompletionRequestMessageContentPartTextArgs::default()
                                .text(format!("{}\n{}\n", self.parameters[0], command_output))
                                .build()?
                                .into(),
                        ])
                        .build()?
                        .into()])
                    .build()?;

                loop {
                    let response: CreateChatCompletionResponse =
                        match client.chat().create(request.clone()).await {
                            std::result::Result::Ok(response) => response,
                            Err(e) => {
                                anyhow::bail!("Failed to execute function: {}", e);
                            }
                        };

                    display_message(
                        Level::ProgramOutput,
                        &format!(
                            "Function executed successfully with result: {}",
                            response.choices[0].clone().message.content.unwrap()
                        ),
                    );

                    display_message(
                        Level::Logging,
                        "Do you want to proceed with this result? (yes/retry/abort)",
                    );

                    let mut user_input = String::new();
                    std::io::stdin()
                        .read_line(&mut user_input)
                        .expect("Failed to read input");
                    let user_input = user_input.trim().to_lowercase();

                    match user_input.as_str() {
                        "yes" => {
                            // Proceed with the result
                            return Ok(response.choices[0].clone().message.content.unwrap());
                        }
                        "retry" => {
                            // Retry the function execution
                            continue;
                        }
                        "abort" => {
                            anyhow::bail!("Execution aborted by the user");
                        }
                        _ => {
                            display_message(
                                Level::Warn,
                                "Invalid input, please enter 'yes', 'retry', or 'abort'.",
                            );
                        }
                    }
                }
            }
        )?;

        return Ok(result);
    }
}
