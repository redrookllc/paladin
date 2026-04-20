use crate::ui;
use console::style;
use futures::StreamExt;
use paladin_ai::{ChatMessage, Client};
use paladin_core::config::Config;
use paladin_core::error::Result;
use std::io::Write;

pub async fn run(symbol: Option<&str>) -> Result<()> {
    let cfg = Config::load()?;
    let client = Client::new(
        cfg.api_base_url.clone(),
        cfg.effective_api_key(),
        cfg.model.clone(),
    )?;
    let mut messages: Vec<ChatMessage> = vec![ChatMessage::system(paladin_ai::prompt::CHAT_SYSTEM_PROMPT)];
    if let Some(sym) = symbol {
        messages.push(ChatMessage::system(format!("Current context: user is looking at {sym}.")));
    }

    println!(
        "{} paladin chat — model {}  (type {} to exit)",
        ui::arrow(),
        style(&cfg.model).cyan(),
        style("/quit").dim()
    );

    loop {
        let prompt = inquire::Text::new(">").prompt();
        let user = match prompt {
            Ok(s) => s,
            Err(_) => break,
        };
        let trimmed = user.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "/quit" || trimmed == "/exit" {
            break;
        }
        messages.push(ChatMessage::user(trimmed));
        let mut stream = client.chat_stream(&messages).await?;
        let mut full = String::new();
        print!("  {} ", ui::ok());
        std::io::stdout().flush().ok();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => {
                    full.push_str(&text);
                    print!("{text}");
                    std::io::stdout().flush().ok();
                }
                Err(e) => {
                    println!("\n  {} {}", ui::error_prefix(), e);
                    break;
                }
            }
        }
        println!();
        messages.push(ChatMessage::assistant(full));
    }
    Ok(())
}
