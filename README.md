# cchain

## Overview
`cchain` is a **modern CLI automation tool** built in Rust that allows you to chain commands together in a structured, retry-aware workflow. Define your automation pipelines in JSON, and let `cchain` handle execution, error recovery, and dynamic input generation using language models (LLMs). Whether you're automating local development tasks, CI/CD pipelines, or complex workflows, `cchain` is designed to replace brittle shell scripts with a **declarative, developer-friendly approach**.

---

## Features
- **Command Chaining**: Execute a series of commands with configurable arguments, retries, and environment variables.
- **Retry Logic**: Automatically retry failed commands with a specified number of attempts.
- **Dynamic Input Generation**: Use LLMs to generate command inputs dynamically (e.g., AI-generated commit messages).
- **Environment Variable Management**: Override environment variables per-command and pass outputs between steps.
- **Interpreter Agnostic**: Run commands in `sh`, `bash`, or any interpreter of your choice.
- **Bookmarking**: Save frequently used command chains for quick access.
- **Lightweight & Fast**: Built in Rust for performance and reliability—no startup lag or dependency hell.

---

## Why `cchain`?
- **Replace Bash Scripts**: Stop debugging flaky shell scripts. Define workflows in JSON for version control, reusability, and auditability.
- **AI-Powered Automation**: Integrate LLMs into your workflows—generate commit messages, summarize logs, or categorize outputs on the fly.
- **Local & CI Consistency**: Test pipeline steps locally before pushing to CI. Ensure identical behavior across environments.
- **Cross-Platform**: Single binary deployment works on Linux, macOS, and Windows.

---

## Comparison to Alternatives
| Feature                  | `cchain`               | Bash Scripts       | Makefiles          | Just              |
|--------------------------|------------------------|--------------------|--------------------|-------------------|
| **Declarative Syntax**   | ✅ JSON                | ❌ Ad-hoc          | ❌ Ad-hoc          | ✅ Custom         |
| **Retry Logic**          | ✅ Built-in            | ❌ Manual          | ❌ Manual          | ❌ Manual         |
| **AI Integration**       | ✅ Native              | ❌ None            | ❌ None            | ❌ None           |
| **Cross-Platform**       | ✅ Single Binary       | ✅ (But Fragile)   | ❌ Limited         | ✅ Single Binary  |
| **Output Chaining**      | ✅ Native              | ❌ Manual          | ❌ Manual          | ❌ Manual         |

---

## Installation

### Cargo
Install `cchain` using Cargo:
```sh
cargo install cchain
```

### Building from Source
Clone the repository and build it using Cargo:
```sh
git clone https://github.com/aspadax/cchain.git
cd cchain
cargo build --release
```

---

## Usage

### Basic Example
Create a JSON configuration file to define your command chain. For example:
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

Run `cchain` with a chain file:
```sh
cchain run --path path/to/cchain_your_file.json
```

### Failure Handling

`cchain` not only supports retry but also additional steps to take when a command line fails to execute. For example, if the LLM call failed when trying to create a commit message, and yo u would like to `git reset` and then perform a new commit after fixing the LLM issue. Then you probably will need this:

```json
[
  {
    "command": "git",
    "arguments": ["add", "--all"],
    "retry": 0
  },
  {
    "command": "git",
    "arguments": ["commit", "-m", "llm_generate('based on the below git diff output, summarize the changes in a few sentences.', 'git --no-pager diff --staged')"],
    "failure_handling_options": {
      "exit_on_failure": true,
      "remedy_command_line": {
        "command": "git",
        "arguments": ["reset"]
      }
    },
    "retry": 0
  },
  {
    "command": "git",
    "arguments": ["push"],
    "retry": 0
  }
]
```

In the second program of this chain, the key `failure_handling_options` provides two options when this program fails to execute. The `exit_on_failure` specifies whether the chain needs to exit entirely when this program fails. The `remedy_command_line` specifies a command to run when the program fails. 

In this case, we need `cchain` to execute `git reset` if the commit fails. And then, we will need to chain to stop running the subsequential programs to prevent a pointless push. 

### Environment Variables
Override environment variables per-command and capture command outputs for reuse:
```json
[
    {
        "command": "echo",
        "arguments": ["$<<env_var_name>>"],
        "environment_variables_override": {
            "hello": "world"
        },
        "stdout_stored_to": "<<env_var_output>>",
        "interpreter": "sh",
        "retry": 0
    },
    {
        "command": "echo",
        "arguments": ["$<<env_var_output>>"],
        "retry": 0
    }
]
```

### AI-Powered Workflows
Use LLMs to generate dynamic inputs. For example, generate a commit message based on `git diff`:
```json
[
    {
        "command": "git",
        "arguments": ["commit", "-m", "llm_generate('generate a commit message', 'git --no-pager diff')"],
        "retry": 1
    }
]
```

Configure your LLM by setting these environment variables:
```sh
export CCHAIN_OPENAI_API_BASE="http://localhost:11434/v1"
export CCHAIN_OPENAI_API_KEY="test_api_key"
export CCHAIN_OPENAI_MODEL="mistral"
```

### Bookmarking
Bookmark frequently used command chains for quick access:
```sh
cchain add path/to/cchain_your_file.json
```
Or, you may want to add a directory of cchain files
```sh
cchain add path/to/a/directory --all
```

Look up existing bookmarked chains and their indexes
```sh
cchain list
```

Run a bookmarked chain by index:
```sh
cchain run 0
```

Delete a bookmark by index:
```sh
cchain remove 0
```

Reset the entire bookmark if needed:
```sh
cchain remove --reset
```

---

## Example Use Cases
- **Git Automation**: Automate `git add`, `git commit` (with AI-generated messages), and `git push` in one workflow.
- **CI/CD Prep**: Run linters, build artifacts, and generate changelogs locally before pushing to CI.
- **Onboarding**: Set up dev environments with a single command—clone repos, install dependencies, and configure tools.
- **AI-Augmented Debugging**: Use LLMs to analyze logs, categorize errors, or suggest fixes.

---

## Contributing
Contributions are welcome! Check out the [Contributing Guidelines](CONTRIBUTING.md) to get started.

---

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---
