extern crate actix;
extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate tokio;
extern crate zmq;

mod client;
mod errors;
mod messenger;
mod server;
mod set;

use actix::Actor;
use clap::{App, Clap};

use errors::ErrorServer;
use messenger::MessengerServer;
use set::SetAgent;

#[derive(Clap)]
enum Subcommands {
    Server(server::Opts),
}

#[derive(Clap)]
#[clap(version = "1.0",
       author = "Jim Berlage <james.berlage@gmail.com>",
       name = "kv",
       about = "Data structure utilities for the shell")]
struct Opts {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[actix_rt::main]
async fn main() {
    let opts = Opts::parse();

    match opts.subcommand {
        Subcommands::Server(opts) => {
            server::start(&opts).await;
        }
    }
}
