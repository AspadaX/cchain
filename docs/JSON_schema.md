# JSON Schema in cchain

By entering the following command, you will get a template that can be worked with:
```bash
cchain new your_file_name
```

This template contains all the available JSON schema elements in `cchain`. Below is a template with comments on what do they do.
```json
[
  {
    "command": "example_command", // Your program's main execution command. For example, in "python main.py","python" is the "command" here.
    "arguments": [ // in "python main.py", "main.py" should be put here.
      "arg1",
      "arg2"
    ],
    "working_directory": "/path/to/work/directory", // The directory where the command will be executed.
    "interpreter": "Sh", // A terminal interpreter to use. If you use `sh`, then put `sh` here. Leaving the field empty or null will disable the interpreter.
    "environment_variables_override": { // An object containing environment variables to override. If you want to override the environment variables, put them here.
      "hello": "world", // This will set hello environment variable to world
      "goodbye": "" // This will set goodbye environment variable to empty string. However, if goodbye has already existed in the real environment variables, it will be overridden.
    },
    "stdout_stored_to": "<<hi>>", // Store the output of the command to a variable named "hi". This can be used in the subsequent commands.
    "stdout_storage_options": {
      "without_newline_characters": true // If set to true, the output will be stored without newline characters.
    },
    "failure_handling_options": {
      "exit_on_failure": true, // If set to true, the program will exit if the command fails.Otherwise, the chain will continue to the next command. 
      "remedy_command_line": { // Set a command to execute when this program fails.
        "command": "remedy_command",
        "arguments": ["arg1", "arg2"]
      }
    },
    "concurrency_group": null, // Set a concurrency group to execute commands concurrently. If set to null, commands will be executed sequentially. Any programs that are in the same concurrency group will be executed concurrently together.
    "retry": 3 // How many times this command is going to be re-executed. `-1` means until success, and `0` means no retry.
  },
  { // This is the next program. It will executed if the previous one finished execution. However, if the program below has the same concurrency group, they will be executed together conurrently.
    "command": "another_command",
    "arguments": [
      "argA",
      "argB"
    ],
    "working_directory": null, // Leave it null to use the current working directory.
    "interpreter": null,
    "environment_variables_override": null,
    "stdout_stored_to": null,
    "stdout_storage_options": {
      "without_newline_characters": true
    },
    "failure_handling_options": {
      "exit_on_failure": true,
      "remedy_command_line": null
    },
    "concurrency_group": null,
    "retry": 5
  }
]
```