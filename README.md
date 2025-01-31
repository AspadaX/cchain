# cchain

## Overview
`cchain` is a command line tool designed to execute a series of commands based on a configuration file. It supports retrying commands if they fail, with a specified number of attempts.

## Features
- Execute commands with specified arguments.
- Retry commands on failure with configurable retry limits.
- Simple configuration using JSON files.
- Logging of command execution and retries.

## Installation
To install `cchain`, clone the repository and build it using Cargo:
```sh
git clone https://github.com/yourusername/cchain.git
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

Run `cchain` with the path to your configuration file:
```sh
./cchain -c path/to/configurations.json
```

## License
This project is licensed under the MIT License.
