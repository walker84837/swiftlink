use clap::{Parser, Subcommand};
use swiftlink_api::{
    BlockingSwiftlinkClient, CreateLinkRequest, CreateLinkResponse, InfoResponse,
    SwiftlinkClientError,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    base_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new short link
    Create {
        /// The URL to shorten
        url: String,
    },
    /// Get information about a short link
    Info {
        /// The code of the short link
        code: String,
    },
    /// Delete a short link
    Delete {
        /// The code of the short link to delete
        code: String,
        /// Bearer token for authentication
        #[arg(short, long)]
        token: String,
    },
}

fn main() -> Result<(), SwiftlinkClientError> {
    let cli = Cli::parse();
    let client = BlockingSwiftlinkClient::new(cli.base_url);

    match &cli.command {
        Commands::Create { url } => {
            let request = CreateLinkRequest { url: url.clone() };
            let response: CreateLinkResponse = client.create_link(&request.url)?;
            println!("Short link created: {}", response.code);
        }
        Commands::Info { code } => {
            let response: InfoResponse = client.get_link_info(code)?;
            println!(
                "Link info for {}: URL = {}, Created At = {}",
                response.code, response.url, response.created_at
            );
        }
        Commands::Delete { code, token } => {
            client.delete_link(code, token)?;
            println!("Link {} deleted.", code);
        }
    }

    Ok(())
}
