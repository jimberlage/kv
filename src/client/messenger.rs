use actix::{Actor, Addr, Context, Handler, Message};
use zmq;

use crate::client::errors::{ErrorServer, SocketConnectionError, SocketOpenError, SocketSendError};

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
        match self.ctx.socket(zmq::SocketType::REQ) {
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

pub enum ChunkError {
    Retry,
    Unexpected,
}

#[derive(Message)]
#[rtype(result = "Result<(), ChunkError>")]
pub struct Chunk(pub Vec<u8>);

impl Handler<Chunk> for MessengerServer {
    type Result = Result<(), ChunkError>;

    fn handle(&mut self, Chunk(data): Chunk, _ctx: &mut Context<Self>) -> Self::Result {
        match self.socket.as_ref().unwrap().send(data, zmq::DONTWAIT) {
            Err(zmq::Error::EAGAIN) => Err(ChunkError::Retry),
            Err(error) => {
                self.error_server_addr.do_send(SocketSendError {
                    error,
                    host: self.host.clone(),
                    port: self.port,
                });
                Err(ChunkError::Unexpected)
            },
            _ => Ok(()),
        }
    }
}
