use actix::{Actor, System};
use clap::Clap;
use tokio::signal::ctrl_c;

pub mod errors;
pub mod messenger;
pub mod stdin;

use errors::ErrorServer;
use messenger::MessengerServer;
use stdin::StdinReaderServer;

#[derive(Clap)]
pub struct Opts {
    #[clap(short, long, default_value = "localhost")]
    host: String,
    #[clap(short, long, default_value = "60054")]
    port: u16,
    #[clap(short, long)]
    name: String,
    #[clap(short, long = "separator", default_value = "\n")]
    sep: String,
}

// TODO: UTF-8 delimiters

pub async fn start(opts: &Opts) {
    let error_server = ErrorServer::new().start();
    let messenger_server =
        MessengerServer::new(&opts.host, opts.port, error_server.clone()).start();
    let stdin_server = StdinReaderServer::new(
        error_server.clone(),
        messenger_server,
        opts.name.clone(),
        opts.sep.clone().into_bytes(),
    )
    .start();

    // TODO: Is panicing appropriate here?
    ctrl_c().await.unwrap();
    System::current().stop();
}
