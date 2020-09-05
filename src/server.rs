/// Starts a persistent server which will give access to the concurrently accessed data structures.
use actix::{Actor, System};
use clap::Clap;
use tokio::signal::ctrl_c;

use errors::ErrorServer;
use messenger::MessengerServer;
use set::SetAgent;

pub mod errors;
pub mod messenger;
pub mod set;

#[derive(Clap)]
pub struct Opts {
    #[clap(short, long, default_value = "localhost")]
    host: String,
    #[clap(short, long, default_value = "60054")]
    port: u16,
}

pub async fn start(opts: &Opts) {
    let error_server = ErrorServer::new().start();
    let set_agent = SetAgent::new().start();
    let messenger_server = MessengerServer::new(&opts.host, opts.port, error_server, set_agent);

    // TODO: Is panicing appropriate here?
    ctrl_c().await.unwrap();
    System::current().stop();
}
