extern crate actix;
extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate zmq;

mod errors;
mod messenger;
mod set;

use actix::Actor;
use clap::App;

use errors::ErrorServer;
use messenger::MessengerServer;
use set::SetAgent;

#[actix_rt::main]
async fn main() {
    let matches = App::new("kv")
        .version("1.0")
        .author("Jim Berlage <james.berlage@gmail.com>")
        .about("Data structure utilities for the shell")
        .subcommand(
            App::new("set")
                .about("Modify a set")
                .subcommand(App::new("json")
                    .about("Return the contents of the set as JSON"))
                .subcommand(App::new("add")
                    .about("Add a value to the set"))
                .subcommand(App::new("contains")
                    .about("Get a value from the set"))
                .subcommand(App::new("remove")
                    .about("Remove a value from the set"))
                .subcommand(App::new("len")
                    .about("Get the length of the set")))
        .subcommand(
            App::new("map")
                .about("Modify a map")
                .subcommand(App::new("json")
                    .about("Return the contents of the map as JSON"))
                .subcommand(App::new("set")
                    .about("Set a value in the map"))
                .subcommand(App::new("get")
                    .about("Get a value from the map"))
                .subcommand(App::new("remove")
                    .about("Remove a value from the map"))
                .subcommand(App::new("len")
                    .about("Get the length of the map")))
        .subcommand(
            App::new("vec")
                .about("Modify a map")
                .subcommand(App::new("json")
                    .about("Return the contents of the vector as JSON"))
                .subcommand(App::new("push")
                    .about("Push a value onto the vector"))
                .subcommand(App::new("pop")
                    .about("Pop a value off the vector"))
                .subcommand(App::new("get")
                    .about("Get a value from the vector at the specified index"))
                .subcommand(App::new("slice")
                    .about("Slice the vector from start to end"))
                .subcommand(App::new("len")
                    .about("Get the length of the vector")))
        .get_matches();

    let error_server = ErrorServer::new().start();
    let set_agent = SetAgent::new().start();
    let messenger_server = MessengerServer::new("localhost", 60054, error_server, set_agent).start();
}
