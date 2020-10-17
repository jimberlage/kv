use actix::{Actor, Addr, Context, Handler};
use prost::Message;
use zmq;

use crate::client::errors::{ErrorServer, SocketConnectionError, SocketOpenError, SocketSendError};
use crate::server::messages as m;

pub struct MessengerServer {
    ctx: zmq::Context,
    host: String,
    port: u16,
    error_server_addr: Addr<ErrorServer>,
    socket: Option<zmq::Socket>,
    id: Option<u32>,
}

impl MessengerServer {
    pub fn new(host: &str, port: u16, error_server_addr: Addr<ErrorServer>) -> Self {
        MessengerServer {
            id: Some(0), // TODO: Need to get an ID somehow.
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

pub enum SetInsertError {
    Retry,
    Unexpected,
}

#[derive(actix::Message)]
#[rtype(result = "Result<(), SetInsertError>")]
pub struct SetInsert {
    pub name: String,
    pub value: Vec<u8>,
}

impl SetInsert {
    fn encode(self, id: u32) -> Vec<u8> {
        let message = m::WireMessage {
            id,
            inner: Some(m::wire_message::Inner::SetInsert(m::SetInsert {
                name: self.name,
                value: self.value,
            })),
        };

        let mut buf = vec![];
        buf.reserve(message.encoded_len());
        message.encode(&mut buf).unwrap();
        buf
    }
}

impl Handler<SetInsert> for MessengerServer {
    type Result = Result<(), SetInsertError>;

    fn handle(&mut self, set_add: SetInsert, _ctx: &mut Context<Self>) -> Self::Result {
        match self
            .socket
            .as_ref()
            .unwrap()
            .send(set_add.encode(self.id.unwrap()), zmq::DONTWAIT) // TODO: Better checking around IDs.
        {
            Err(zmq::Error::EAGAIN) => Err(SetInsertError::Retry),
            Err(error) => {
                self.error_server_addr.do_send(SocketSendError {
                    error,
                    host: self.host.clone(),
                    port: self.port,
                });
                Err(SetInsertError::Unexpected)
            }
            _ => Ok(()),
        }
    }
}
