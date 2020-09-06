use actix::{Actor, System};
use clap::Clap;
use tokio::signal::ctrl_c;

pub mod errors;
pub mod messenger;
pub mod stdin;

use errors::ErrorServer;
use messenger::MessengerServer;

#[derive(Clap)]
pub struct Opts {
    #[clap(short, long, default_value = "localhost")]
    host: String,
    #[clap(short, long, default_value = "60054")]
    port: u16,
}

// TODO: UTF-8 delimiters

pub async fn start(opts: &Opts) {
    let error_server = ErrorServer::new().start();
    let messenger_server = MessengerServer::new(&opts.host, opts.port, error_server);

    // TODO: Is panicing appropriate here?
    ctrl_c().await.unwrap();
    System::current().stop();
}
