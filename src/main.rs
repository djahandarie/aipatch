use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use async_openai::{
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Path to the file to be processed
    #[arg(index = 1)]
    file: PathBuf,

    /// Prompt to provide to the LLM
    #[arg(index = 2)]
    prompt: String,

    /// Model to use for the request
    #[arg(short, long, default_value = "gpt-4o-mini")]
    model: String,

    /// Do not run `git add --patch` at the end (for example, if you want to inspect changes within your IDE)
    #[arg(short, long)]
    no_patch: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let git_status = std::process::Command::new("git")
        .args(&["ls-files", "--error-unmatch", cli.file.to_str().unwrap()])
        .output();

    if let Ok(output) = git_status {
        if !output.status.success() {
            if !confirm_continue("\x1b[31mWARNING: FILE NOT TRACKED BY GIT. The file will be OVERWRITTEN with no way to revert it. Do you want to continue? (y/N) \x1b[0m")? {
                return Ok(());
            }
        }
    } else {
        if !confirm_continue("\x1b[33mWARNING: Unable to determine git status. The file may be overwritten with no way to revert it. Do you want to continue? (y/N) \x1b[0m")? {
            return Ok(());
        }
    }
    let original_content =
        fs::read_to_string(&cli.file).with_context(|| "Could not read the input file")?;

    let edited_content = request_openai_edit(&original_content, &cli.prompt, &cli.model).await?;

    fs::write(&cli.file, [&edited_content, "\n"].concat())
        .with_context(|| "Failed to write updated content to file")?;

    if !cli.no_patch {
        let git_add = std::process::Command::new("git")
            .args(&["add", "--patch", cli.file.to_str().unwrap()])
            .status()
            .context("Failed to run git add --patch")?;
        if !git_add.success() {
            return Err(anyhow::anyhow!("git add --patch failed"));
        }
    }

    Ok(())
}

async fn request_openai_edit(original_text: &str, prompt: &str, model: &str) -> Result<String> {
    let client = Client::new();

    let mut messages = vec![];
    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(format!(
                "You are used within a CLI tool, you accept a prompt and file contents from the user. Generate new file contents based on the instructions in the prompt. Return only the revised file contents. DO NOT WRAP CODE IN BACKTICKS.\n\nPrompt:\n{}\n\nOriginal file:\n{}\n",
                prompt, original_text
            ))
            .build()?
            .into(),
    );

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .build()?;

    println!("{}", serde_json::to_string(&request).unwrap());

    let mut response = client.chat().create(request).await?;

    response
        .choices
        .pop()
        .ok_or_else(|| anyhow::anyhow!("No choices in response"))?
        .message
        .content
        .ok_or_else(|| anyhow::anyhow!("No content in choice"))
}

fn confirm_continue(msg: &str) -> Result<bool> {
    print!("{}", msg);
    io::stdout().flush()?;

    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;
    if !matches!(choice.trim().to_lowercase().as_str(), "y" | "yes") {
        println!("Operation cancelled.");
        return Ok(false);
    }
    Ok(true)
}
