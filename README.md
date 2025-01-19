# aipatch

**aip** (aipatch) is a command-line tool that utilizes Large Language Models (LLMs) to process files based on user-provided prompts. It streamlines the process of applying AI-driven changes to your code or text files directly from the terminal.

By specifying a file and a prompt, **aip** communicates with an LLM to process the content of the file according to your instructions. This tool can assist with tasks such as code refactoring, content rewriting, summarization, and more.

## Usage

Usage: `aip --help`

## Examples

- **Apply changes to a file using a prompt:**

  `aip example.py "Refactor the code to use async functions"`

- **Specify a different model:**

  `aip --model o1-mini document.txt "Improve the tone and flow"`

- **Process a file without staging changes:**

  `aip README.md "Fix typos and grammatical errors" --no-patch`
