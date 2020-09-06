use actix::{Actor, Addr, Context, Handler};
use prost::Message;
use zmq;

use crate::client::errors::{ErrorServer, SocketConnectionError, SocketOpenError, SocketSendError};
use crate::messages as m;

pub struct MessengerServer {
    ctx: zmq::Context,
    host: String,
    port: u16,
    error_server_addr: Addr<ErrorServer>,
    socket: Option<zmq::Socket>,
}

impl MessengerServer {
    pub fn new(host: &str, port: u16, error_server_addr: Addr<ErrorServer>) -> Self {
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

pub enum SetAddError {
    Retry,
    Unexpected,
}

#[derive(actix::Message)]
#[rtype(result = "Result<(), SetAddError>")]
pub struct SetAdd {
    pub name: String,
    pub data: Vec<u8>,
}

impl SetAdd {
    fn encode(self) -> Vec<u8> {
        let message = m::WireMessage {
            inner: Some(m::wire_message::Inner::SetAdd(m::SetAdd {
                name: self.name,
                data: self.data,
            })),
        };

        let mut buf = vec![];
        buf.reserve(message.encoded_len());
        message.encode(&mut buf).unwrap();
        buf
    }
}

impl Handler<SetAdd> for MessengerServer {
    type Result = Result<(), SetAddError>;

    fn handle(&mut self, set_add: SetAdd, _ctx: &mut Context<Self>) -> Self::Result {
        match self.socket.as_ref().unwrap().send(set_add.encode(), zmq::DONTWAIT) {
            Err(zmq::Error::EAGAIN) => Err(SetAddError::Retry),
            Err(error) => {
                self.error_server_addr.do_send(SocketSendError {
                    error,
                    host: self.host.clone(),
                    port: self.port,
                });
                Err(SetAddError::Unexpected)
            }
            _ => Ok(()),
        }
    }
}
