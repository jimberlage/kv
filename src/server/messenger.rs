use std::io::Cursor;

use actix::prelude::SendError;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use prost::Message;
use zmq;

use crate::messages as m;
use crate::server::errors::{
    ErrorServer, MessageDecodeError, SocketConnectionError, SocketOpenError, SocketRecvError,
};
use crate::server::set::{self, Insert, SetAgent};

pub struct MessengerServer {
    ctx: zmq::Context,
    host: String,
    port: u16,
    error_server_addr: Addr<ErrorServer>,
    set_agent_addr: Addr<SetAgent>,
    socket: Option<zmq::Socket>,
}

impl MessengerServer {
    pub fn new(
        host: &str,
        port: u16,
        error_server_addr: Addr<ErrorServer>,
        set_agent_addr: Addr<SetAgent>,
    ) -> Self {
        MessengerServer {
            ctx: zmq::Context::new(),
            host: host.to_owned(),
            port,
            error_server_addr,
            set_agent_addr,
            socket: None,
        }
    }
}

impl Actor for MessengerServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        match self.ctx.socket(zmq::SocketType::ROUTER) {
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

impl Handler<set::Insert> for MessengerServer {
    type Result = bool;

    fn handle(&mut self, insert: Insert, ctx: &mut Context<Self>) -> Self::Result {
        match self.set_agent_addr.try_send(insert) {
            Err(SendError::Closed(insert)) => {
                ctx.address().do_send(insert);
                false
            }
            Err(SendError::Full(insert)) => {
                ctx.address().do_send(insert);
                false
            }
            Ok(()) => true,
        }
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct Recv;

impl Handler<Recv> for MessengerServer {
    type Result = ();

    fn handle(&mut self, _: Recv, ctx: &mut Context<Self>) -> Self::Result {
        match self.socket.as_ref().unwrap().recv_bytes(zmq::DONTWAIT) {
            Err(zmq::Error::EAGAIN) => (),
            Err(error) => {
                self.error_server_addr.do_send(SocketRecvError {
                    error,
                    host: self.host.clone(),
                    port: self.port,
                });
            }
            Ok(bytes) => match m::WireMessage::decode(Cursor::new(&bytes)) {
                Ok(m::WireMessage {
                    inner: Some(m::wire_message::Inner::SetInsert(m::SetInsert { name, value })),
                }) => ctx.address().do_send(Insert { name, value }),
                Ok(m::WireMessage { inner: None }) => {
                    self.error_server_addr
                        .do_send(MessageDecodeError(None, bytes));
                }
                Err(decode_error) => {
                    self.error_server_addr
                        .do_send(MessageDecodeError(Some(decode_error), bytes));
                }
            },
        };

        ctx.address().do_send(Recv);
    }
}
