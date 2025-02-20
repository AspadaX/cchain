# Setup LLM

You may also use LLMs to generate the chain files:

```bash
cchain new print_hello_world --prompt "print a hello world in chinese on the screen, then print hello world in russian."
```

Configure your LLM by setting these environment variables:
```sh
export CCHAIN_OPENAI_API_BASE="http://localhost:11434/v1"
export CCHAIN_OPENAI_API_KEY="test_api_key"
export CCHAIN_OPENAI_MODEL="mistral"
```