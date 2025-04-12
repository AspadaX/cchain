# 🚀 cchain 

**Automate like the future depends on it.**  
*Replace brittle shell scripts with AI-powered, retry-aware workflows. Built in Rust for speed and reliability.*

---

## ⚡ Quick Example

**Automate Git commits with AI-generated messages** *(no more "fix typo" commits!)*:  
```bash
cchain run ./cchain_git_commit.json # Using a pre-built workflow to commit changes with AI
```

**JSON Workflow** (`cchain_git_commit.json`):
```json
[
  {
    "command": "git",
    "arguments": ["add", "--all"],
    "retry": 3
  },
  {
    "command": "git",
    "arguments": [
      "commit", "-m", 
      "llm_generate('Summarize these changes in 1 line', 'git diff --staged')"
    ],
    "failure_handling_options": {
      "exit_on_failure": true,
      "remedy_command_line": { "command": "git", "arguments": ["reset"] }
    }
  },
  {
    "command": "git",
    "arguments": ["push"],
    "retry": 2
  }
]
```

---

## 🌟 Why Developers Love `cchain`

| **Problem**                          | **cchain Solution**                                      |
|--------------------------------------|----------------------------------------------------------|
| "Bash scripts break at 3 AM"         | ✅ Declarative JSON workflows with built-in **retries**  |
| "Commit messages take forever"       | ✅ **AI-generated inputs** via LLMs                      |
| "Why does CI/CD fail locally?!"      | ✅ **Identical behavior** across local/CI environments   |
| "Makefiles are so 1980"              | ✅ Simple syntax with **concurrency** (beta)             |
| "Dependency hell"                    | ✅ Single binary—**zero runtime dependencies**           |

---

## 🛠️ Features

### 🔗 Chain Commands Like a Pro
- Retry failed steps (up to `N` times, or use `-1` to retry indefinitely until succeeded)
- Pass outputs between commands via environment variables
- **Fix failures automatically**: Roll back with `remedy_command_line`

### 🤖 AI-Powered Automation
```json
"arguments": ["llm_generate('Summarize this error', 'cat crash.log')"]
```
- Integrate LLMs (OpenAI, local models via Ollama, etc.)
- Generate commit messages, error summaries, test data on the fly

### 🌐 Cross-Platform Consistency
- Works on **Linux/macOS/Windows** out of the box
- No more `if [[ "$OSTYPE" == "linux-gnu"* ]]; then...`

### ⚡ Performance You’ll Notice
- Built in Rust—starts faster than your shell’s `&&` chain
- Uses 10x less memory than Python/Ruby scripts

---

## 📦 Installation

### Cargo
You need to ensure that you have Rust installed on your system. Otherwise, you may need to follow the instructions here to install Rust: https://www.rust-lang.org/learn/get-started

If you are using a Debian system, you will need to have `build-essentials` installed. Below is an example for Ubuntu: 
```bash
sudo apt update
sudo apt install build-essential
```

Install the tools. 
```bash
cargo install cchain
```

### Homebrew
You will need to tap `cchain`'s homebrew repo first. 
```bash
brew tap AspadaX/cchain
```
Then, you can install the package:
```bash
brew install cchain
```

### Pre-built binaries
You can navigate to the `release` section for downloading the latest binaries available. 

---

## 🚀 Getting Started

### 1. Create Your First Workflow
```bash
cchain new deploy --prompt "Create a workflow to pull docker image from xxx, then run it in the background"
```
*AI generates a starter `cchain_deploy.json`!*

### 2. Run It!
```bash
cchain run ./cchain_deploy.json
```

### 3. Save for Later
```bash
cchain add ./cchain_deploy.json  # Bookmark it as workflow #0
cchain run 0  # Re-run anytime
cchain run deploy # Or, use keyword to run it
cchain run "deploy some other fancy stuff" # Or, use multiple keywords
```
### 4. Access Public Chains
You may also want to share your chain, or find chains created by someone else. I hosted a GitHub repository for this purpose:

```bash
git clone https://github.com/AspadaX/cchain-chains
```

This respository can be directly addded to your local bookmark:

```bash
cchain add https://github.com/AspadaX/cchain-chains
```

It is much welcomed to PR new chains to this repository!

---

## 🧩 Advanced Usage

### Dynamic Environment Variables
```json
{
  "command": "echo",
  "arguments": ["Building $APP_VERSION"],
  "environment_variables_override": {
    "APP_VERSION": "llm_generate('Generate a semantic version')"
  },
  "stdout_stored_to": "<<build_id>>"  # Pass to next command!
}
```

### Concurrent Tasks (Beta)
```json
[
  {
    "command": "xh download http://example.com/large-asset.zip",
    "concurrency_group": 1
  },
  {
    "command": "xh download http://example.com/large-asset.zip",
    "concurrency_group": 1
  },
  {
    "command": "xh download http://example.com/large-asset.zip",
    "concurrency_group": 1
  }
]  # Download 3 files in parallel
```

You may find examples in the `./examples` directory of this repo. Also, you may use the following command to generate a template chain file:
```bash
cchain new your_file_name
```

---

## 🔍 Comparison

|                      | `cchain`       | Bash           | Just           | Python         |
|----------------------|----------------|----------------|----------------|----------------|
| **Retry Logic**      | ✅ Built-in    | ❌ Manual      | ❌ Manual      | ❌ Manual      |
| **AI Integration**   | ✅ Native      | ❌ None        | ❌ None        | ❌ Add-ons     |
| **Cross-Platform**   | ✅ Single Bin | ✅ (Fragile)  | ✅             | ✅ (If setup)  |
| **Learning Curve**   | Low (JSON)     | High           | Medium         | High           |

---

## 🛠️ Use Cases (Just mocks, but feasible)

### CI/CD Made Simple
```json
[
  { "command": "cargo test", "retry": 2 },
  { "command": "llm_generate('Write release notes', 'git log') > CHANGELOG.md" },
  { "command": "docker build -t myapp ." }
]
```

### Developer Onboarding
```bash
cchain new setup --prompt "Clone repo, install deps, start services"
```

### AI-Augmented Debugging
```json
{
  "command": "llm_generate('Fix this error', './failing_script.sh 2>&1')",
  "stdout_stored_to": "<<fix_suggestion>>"
}
```

---

## 📚 Documentation

**Guides**  
- [Full JSON Schema](docs/JSON_schema.md)
- [LLM Configuration](docs/LLM_setup.md)

---

## 🤝 Contributing

We welcome PRs!  

---

## 📜 License

MIT © 2024 Xinyu Bao
*"Do whatever you want—just don’t make CI pipelines cry."*
