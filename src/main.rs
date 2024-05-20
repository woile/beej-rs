use beej_rs::{
    cli::{Cli, Commands},
    examples,
};

use clap::Parser;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ShowIp {
            host,
            family,
            service,
        } => examples::showip(host, family, service),
        Commands::StreamServer => examples::streamserver(),
        Commands::StreamClient { host } => examples::streamclient(host),
        Commands::SocketListener { port, family } => examples::socketlistener(port, family),
        Commands::SocketTalker {
            host,
            port,
            message,
        } => examples::sockettalker(host, port, message),
        Commands::PollStdIn => examples::pollstdin(),
        Commands::PollServer { port } => examples::pollserver(port),
        Commands::Select => examples::select(),
        Commands::SelectServer { port } => examples::select_server(port),
        Commands::Broadcaster {
            host,
            port,
            message,
        } => examples::broadcaster(host, port, message),
    }
}
