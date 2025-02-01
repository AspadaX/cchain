# cchain

## Overview
`cchain` is a command line tool designed to execute a series of commands based on a configuration file. It supports retrying commands if they fail, with a specified number of attempts.

## Features
- Execute commands with specified arguments.
- Retry commands on failure with configurable retry limits.
- Simple configuration using JSON files.
- Logging of command execution and retries.

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

## License
This project is licensed under the MIT License.
