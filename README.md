# cchain

## Overview
`cchain` is a command line tool designed to execute a series of commands based on a configuration file. It supports retrying commands if they fail, with a specified number of attempts. Additionally, `cchain` can generate command inputs using a language model (LLM) based on specified functions.

## Features
- Execute commands with specified arguments.
- Retry commands on failure with configurable retry limits.
- Simple configuration using JSON files.
- Logging of command execution and retries.
- Generate command inputs dynamically using LLM functions.

## Installation

### Cargo
Use Cargo to install `cchain`:
```sh
cargo install cchain
```

### Building from Source
To install `cchain`, clone the repository and build it using Cargo:
```sh
git clone https://github.com/aspadax/cchain.git
cd cchain
cargo build --release
```

## Usage
Create a JSON configuration file with the commands you want to execute. Example configuration:
```json
[
    {
        "command": "echo",
        "arguments": ["Hello, world!"],
        "retry": 3
    },
    {
        "command": "ls",
        "arguments": ["-la"],
        "retry": 1
    }
]
```
Additionally, if you do not specify a configuration file, `cchain` will list all available configuration files in the current working directory that start with `cchain_` and have a `.json` extension. You can then select the desired configuration file by entering the corresponding number.

Example:
```sh
cchain
```
This will prompt you to select from the available configuration files in the current directory.

Run `cchain` with the path to your configuration file:
```sh
cchain --configuration_file path/to/configuration.json
```

Also, if you would like to pick a command chain in a different folder than the current one, you can use the `--directory` flag:
```sh
cchain --configuration_files path/to/the/directory
```

To generate a template configuration file, use the `--generate` flag:
```sh
cchain --generate
```

### Using Functions with LLM
You can specify functions in your configuration file that will generate command inputs dynamically using a language model. Example configuration with a function:
```json
[
    {
        "command": "echo",
        "arguments": ["Hello, world!"],
        "retry": 3
    },
    {
        "command": "git",
        "arguments": ["commit", "-m", "llm_generate('generate a commit message', 'git --no-pager diff')"],
        "retry": 1
    }
]
```
In this example, the `llm_generate` function will use the specified arguments to generate a git commit message by prompting the LLM with `git --no-pager diff`.

You can configure the LLM by setting the following environment variables:
```sh
export CCHAIN_OPENAI_API_BASE="http://localhost:11434/v1"
export CCHAIN_OPENAI_API_KEY="test_api_key"
export CCHAIN_OPENAI_MODEL="mistral"
```
Here in the example, we are using a locally hosted Ollama model. 

## License
This project is licensed under the MIT License.
