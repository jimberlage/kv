use actix::{Actor, Addr, Context};
use zmq;

use crate::client::errors::{ErrorServer, SocketConnectionError, SocketOpenError};

pub struct MessengerServer {
    ctx: zmq::Context,
    host: String,
    port: u16,
    error_server_addr: Addr<ErrorServer>,
    socket: Option<zmq::Socket>,
}

impl MessengerServer {
    pub fn new(
        host: &str,
        port: u16,
        error_server_addr: Addr<ErrorServer>,
    ) -> Self {
        MessengerServer {
            ctx: zmq::Context::new(),
            host: host.to_owned(),
            port,
            error_server_addr,
            socket: None,
        }
    }
}

// TODO: Need to get logic around collisions.
// Client sends UUID + timestamp as connection ID.

impl Actor for MessengerServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        match self.ctx.socket(zmq::SocketType::SUB) {
            Err(error) => self.error_server_addr.do_send(SocketOpenError(error)),
            Ok(socket) => {
                self.socket = Some(socket);
                if let Err(error) = self
                    .socket
                    .as_ref()
                    .unwrap()
                    .connect(&format!("tcp://{}:{}", self.host, self.port))
                {
                    self.error_server_addr.do_send(SocketConnectionError {
                        error,
                        host: self.host.clone(),
                        port: self.port,
                    })
                }
            }
        }
    }
}
