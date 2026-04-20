use crate::ui;
use crate::Cli;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use paladin_core::error::{Error, Result};
use std::io;

pub fn run(shell: Option<Shell>, print: bool) -> Result<()> {
    let shell = match shell {
        Some(s) => s,
        None => detect_shell().ok_or_else(|| Error::InvalidInput("could not detect shell; pass one explicitly".into()))?,
    };
    let mut cmd = Cli::command();
    if print {
        generate(shell, &mut cmd, "paladin", &mut io::stdout());
    } else {
        generate(shell, &mut cmd, "paladin", &mut io::stdout());
        eprintln!("  {} generated {} completions (redirect to your shell dir)", ui::ok(), shell);
    }
    Ok(())
}

fn detect_shell() -> Option<Shell> {
    let sh = std::env::var("SHELL").ok()?;
    if sh.ends_with("zsh") {
        Some(Shell::Zsh)
    } else if sh.ends_with("bash") {
        Some(Shell::Bash)
    } else if sh.ends_with("fish") {
        Some(Shell::Fish)
    } else {
        None
    }
}
