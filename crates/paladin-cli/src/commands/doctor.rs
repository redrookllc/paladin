use crate::ui;
use console::style;
use paladin_ai::{ChatMessage, Client};
use paladin_core::config::Config;
use paladin_core::error::Result;
use paladin_core::paths;

pub async fn run(_fix: bool) -> Result<()> {
    let cfg = Config::load()?;
    ui::section("paths");
    println!("  home   {}", paths::home_dir()?.display());
    println!("  config {}", paths::config_path()?.display());
    println!("  cache  {}", paths::cache_dir()?.display());
    println!("  logs   {}", paths::logs_dir()?.display());

    ui::section("api");
    println!("  base   {}", style(&cfg.api_base_url).cyan());
    println!("  model  {}", style(&cfg.model).cyan());
    let has_key = cfg.effective_api_key().is_some();
    println!(
        "  key    {}",
        if has_key {
            style("present".to_string()).green()
        } else {
            style("missing".to_string()).red()
        }
    );

    ui::section("connectivity");
    let sp = ui::spinner("pinging model");
    let client = Client::new(cfg.api_base_url.clone(), cfg.effective_api_key(), cfg.model.clone())?;
    let res = client
        .chat(&[
            ChatMessage::system("You are a health check."),
            ChatMessage::user("reply with the single word: ok"),
        ])
        .await;
    sp.finish_and_clear();
    match res {
        Ok(text) => println!("  {} reachable — {}", ui::ok(), style(text.trim()).dim()),
        Err(e) => println!("  {} {}", ui::fail(), e),
    }

    ui::section("market data");
    let sp = ui::spinner("fetching SPY");
    let res = paladin_data::fetch_history("SPY", paladin_core::candle::Interval::OneDay, Some("5d")).await;
    sp.finish_and_clear();
    match res {
        Ok(candles) => println!("  {} yahoo reachable — {} candles", ui::ok(), candles.len()),
        Err(e) => println!("  {} {}", ui::fail(), e),
    }
    Ok(())
}
