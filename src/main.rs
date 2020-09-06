extern crate actix;
extern crate actix_rt;
extern crate base64;
extern crate clap;
extern crate log;
extern crate prost;
extern crate prost_types;
extern crate simple_logger;
extern crate tokio;
extern crate zmq;

mod client;
mod server;
pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}

use clap::Clap;

#[derive(Clap)]
enum Subcommands {
    Client(client::Opts),
    Server(server::Opts),
}

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Jim Berlage <james.berlage@gmail.com>",
    name = "kv",
    about = "Data structure utilities for the shell"
)]
struct Opts {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[actix_rt::main]
async fn main() {
    let opts = Opts::parse();

    match opts.subcommand {
        Subcommands::Client(opts) => {
            client::start(&opts).await;
        }
        Subcommands::Server(opts) => {
            server::start(&opts).await;
        }
    }
}
