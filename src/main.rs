mod blockchain;
mod decoder;
mod jinja;
mod tracer;

use std::collections::HashMap;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Trace
    Trace(TraceArgs),

    /// Decode
    Decode(DecodeArgs),
}

#[derive(Args)]
struct TraceArgs {
    #[command(subcommand)]
    command: TraceCommands,
}

#[derive(Subcommand)]
enum TraceCommands {
    /// Trace message
    Message(TraceMessageArgs),
}

#[derive(Args)]
struct TraceMessageArgs {
    /// Path to dir with ABIs for decoding messages
    #[arg(long)]
    abi_dir: String,

    /// Message id
    #[arg(long)]
    message_id: String,

    /// Decode messages
    #[arg(long)]
    decode: bool,

    /// Gosh endpoints url
    #[arg(long, default_value_t = String::from("https://network.gosh.sh"))]
    endpoints: String,

    /// Gosh explorer url
    #[arg(long, default_value_t = String::from("https://gosh.live"))]
    explorer: String,
}

#[derive(Args)]
struct DecodeArgs {
    #[command(subcommand)]
    command: DecodeCommands,
}

#[derive(Subcommand)]
enum DecodeCommands {
    /// Decode account data
    Account(DecodeAccountArgs),

    /// Decode message
    Message(DecodeMessageArgs),
}

#[derive(Args)]
struct DecodeAccountArgs {
    /// Path to dir with ABIs for decoding messages
    #[arg(long)]
    abi_dir: String,

    /// Message id
    #[arg(long)]
    account_id: String,

    /// Gosh endpoints url
    #[arg(long, default_value_t = String::from("https://network.gosh.sh"))]
    endpoints: String,
}

#[derive(Args)]
struct DecodeMessageArgs {
    /// Path to dir with ABIs for decoding messages
    #[arg(long)]
    abi_dir: String,

    /// Message id
    #[arg(long)]
    message_id: String,

    /// Gosh endpoints url
    #[arg(long, default_value_t = String::from("https://network.gosh.sh"))]
    endpoints: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Trace message command
    match &cli.commands {
        Commands::Trace(args) => match &args.command {
            TraceCommands::Message(args) => {
                let abis = blockchain::get_abi_files(&args.abi_dir);
                let context = blockchain::get_client_context(vec![args.endpoints.clone()])
                    .map_err(|e| anyhow::format_err!("Error creating context {e}"))?;
                let trace = tracer::trace_message(&context, &abis, &args.message_id, &args.decode)
                    .await
                    .map_err(|e| anyhow::format_err!("Error tracing message {e}"))?;

                let mut kwargs: HashMap<&str, String> = HashMap::new();
                kwargs.insert("explorer_url", args.explorer.clone());
                tracer::render_trace_template(&trace, Some(kwargs))
                    .map_err(|e| anyhow::format_err!("Error rendering output {e}"))?;
            }
        },
        Commands::Decode(args) => match &args.command {
            DecodeCommands::Account(args) => {
                let abis = blockchain::get_abi_files(&args.abi_dir);
                let context = blockchain::get_client_context(vec![args.endpoints.clone()])
                    .map_err(|e| anyhow::format_err!("Error creating context {e}"))?;
                let account = decoder::decode_account(&context, &abis, &args.account_id)
                    .await
                    .map_err(|e| anyhow::format_err!("Error decoding account {e}"))?;
                decoder::render_account(&account)
                    .map_err(|e| anyhow::format_err!("Error rendering account {e}"))?;
            }
            DecodeCommands::Message(args) => {
                let abis = blockchain::get_abi_files(&args.abi_dir);
                let context = blockchain::get_client_context(vec![args.endpoints.clone()])
                    .map_err(|e| anyhow::format_err!("Error creating context {e}"))?;
                let message = decoder::decode_message(&context, &abis, &args.message_id)
                    .await
                    .map_err(|e| anyhow::format_err!("Error decoding message {e}"))?;
                decoder::render_message(&message)
                    .map_err(|e| anyhow::format_err!("Error rendering message {e}"))?;
            }
        },
    }
    Ok(())
}
