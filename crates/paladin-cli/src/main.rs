mod commands;
mod logging;
mod ui;

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use paladin_core::error::{Error, Result};

#[derive(Parser)]
#[command(name = "paladin")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Paladin — desktop market analytics & LLM-backed signal generation")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true, help = "Assume yes for all prompts")]
    yes: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Launch the desktop GUI")]
    Gui,

    #[command(about = "Generate a trade signal for a symbol")]
    Signal {
        symbol: String,
        #[arg(long, default_value = "1d")]
        interval: String,
        #[arg(long, help = "Yahoo range string (e.g. 1mo, 6mo, 1y, 2y)")]
        range: Option<String>,
        #[arg(long, help = "Emit raw JSON")]
        json: bool,
    },

    #[command(about = "Render a terminal chart + indicator summary")]
    Chart {
        symbol: String,
        #[arg(long, default_value = "1d")]
        interval: String,
    },

    #[command(about = "Streaming chat REPL backed by an OpenAI-compatible endpoint")]
    Chat {
        #[arg(long, help = "Optional symbol context")]
        symbol: Option<String>,
    },

    #[command(about = "Manage cached market data")]
    Data {
        #[command(subcommand)]
        action: DataAction,
    },

    #[command(about = "View or edit configuration")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(about = "Check API + data reachability")]
    Doctor {
        #[arg(long)]
        fix: bool,
    },

    #[command(about = "Install shell completions")]
    Completions {
        #[arg(value_enum)]
        shell: Option<Shell>,
        #[arg(long)]
        print: bool,
    },

    #[command(about = "Update paladin itself")]
    Update {
        #[arg(long = "self")]
        update_self: bool,
    },
}

#[derive(Subcommand)]
enum DataAction {
    #[command(about = "Fetch and cache history for a symbol")]
    Fetch {
        symbol: String,
        #[arg(long, default_value = "1d")]
        interval: String,
    },
    #[command(about = "Clear all cached data")]
    Clear,
}

#[derive(Subcommand)]
enum ConfigAction {
    #[command(about = "Show the active configuration")]
    Show,
    #[command(about = "Print the config file path")]
    Path,
    #[command(about = "Set a config key")]
    Set { key: String, value: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = logging::init(cli.verbose) {
        eprintln!("{} {}", ui::error_prefix(), e);
    }

    let result: Result<()> = match cli.command {
        Commands::Gui => commands::gui::run(),
        Commands::Signal { symbol, interval, range, json } => {
            commands::signal::run(&symbol, &interval, range.as_deref(), json).await
        }
        Commands::Chart { symbol, interval } => commands::chart::run(&symbol, &interval).await,
        Commands::Chat { symbol } => commands::chat::run(symbol.as_deref()).await,
        Commands::Data { action } => match action {
            DataAction::Fetch { symbol, interval } => commands::data::fetch(&symbol, &interval).await,
            DataAction::Clear => commands::data::clear(),
        },
        Commands::Config { action } => match action {
            ConfigAction::Show => commands::config::show(),
            ConfigAction::Path => commands::config::path(),
            ConfigAction::Set { key, value } => commands::config::set(&key, &value),
        },
        Commands::Doctor { fix } => commands::doctor::run(fix).await,
        Commands::Completions { shell, print } => commands::completions::run(shell, print),
        Commands::Update { update_self } => commands::update::run(update_self).await,
    };

    if let Err(e) = result {
        match e {
            Error::NoData(sym) => eprintln!("{} no data for {}", ui::error_prefix(), sym),
            other => eprintln!("{} {}", ui::error_prefix(), other),
        }
        std::process::exit(1);
    }
}
