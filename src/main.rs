extern crate actix;
extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate tokio;
extern crate zmq;

mod client;
mod server;

use clap::Clap;

#[derive(Clap)]
enum Subcommands {
    Client(client::Opts),
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
        Subcommands::Client(_opts) => {
            unimplemented!();
        }
        Subcommands::Server(opts) => {
            server::start(&opts).await;
        }
    }
}
