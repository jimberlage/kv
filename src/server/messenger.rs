use std::io::Cursor;

use actix::prelude::SendError;
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler};
use prost::Message;
use zmq;

use crate::client::messages as cm;
use crate::server::messages as m;
use crate::server::errors::{
    ErrorServer, MessageDecodeError, SocketConnectionError, SocketOpenError, SocketRecvError, UnsentResponseError,
};
use crate::server::set::{self, SetAgent};

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

    fn handle(&mut self, insert: set::Insert, ctx: &mut Context<Self>) -> Self::Result {
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
enum Error {
    InvalidMessage { id: u32 },
}

impl Handler<Error> for MessengerServer {
    type Result = ();

    fn handle(&mut self, error: Error, ctx: &mut Context<Self>) -> Self::Result {
        match error {
            Error::InvalidMessage { id } => match &self.socket {
                Some(socket) => {
                    let mut buf = vec![];
                    cm::WireMessage {
                        id,
                        inner: Some(cm::wire_message::Inner::UnrecognizedMessageError(cm::UnrecognizedMessageError{})),
                    }.encode(&mut buf);

                    match socket.send(buf, 0) {
                        // TODO: What should happen here?
                        Err(error) => {},
                        Ok(()) => (),
                    };
                },
                // Log that a client would not have received a response to their request.
                // Not much more we can do there.  Clients should have a timeout due to the
                // possibility of encountering an error like this.
                None => self.error_server_addr.do_send(UnsentResponseError {
                    client_id: id,
                    host: self.host.clone(),
                    port: self.port,
                }),
            },
        }
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct Recv;

impl Handler<Recv> for MessengerServer {
    type Result = ();

    // The main read loop of this actor.  Waits for incoming messages over the zeromq socket, and
    // dispatches actions to the data structure agents and sends responses to connected clients on
    // errors.
    fn handle(&mut self, _: Recv, ctx: &mut Context<Self>) -> Self::Result {
        // See http://api.zeromq.org/master:zmq-recv for an overview of error types.
        match self.socket.as_ref().unwrap().recv_bytes(zmq::DONTWAIT) {
            // EAGAIN, with the DONTWAIT flag set, indicates that there is no data.  This is not
            // considered an error.
            Err(zmq::Error::EAGAIN) => (),
            // If the zmq process was interrupted with a signal, retry; if the signal should kill
            // this process too, we'll know soon enough.
            Err(zmq::Error::EINTR) => (),
            // ETERM/ENOTSOCK: The context was terminated, or something got the socket into a bad
            // state; the actor must be restarted for messages to be received properly.
            // Other errors should be handled as an exceptional event; still, let it crash.
            Err(error) => {
                self.error_server_addr.do_send(SocketRecvError {
                    error,
                    host: self.host.clone(),
                    port: self.port,
                });

                ctx.stop();
            }
            // Received a message; try to deserialize it according to our message format.
            Ok(bytes) => match m::WireMessage::decode(Cursor::new(&bytes)) {
                // Queue up a message to tell us to insert the given data.
                Ok(m::WireMessage {
                    id,
                    inner: Some(m::wire_message::Inner::SetInsert(m::SetInsert { name, value })),
                }) => ctx.address().do_send(set::Insert { id, name, value }),
                // No idea what context this would happen in, but it's not fatal at all.
                // Log the error and move on.
                Ok(m::WireMessage { id, inner: None }) => {
                    self.error_server_addr
                        .do_send(MessageDecodeError(None, bytes));

                    // TODO: Send a failure message to the server.
                }
                // A message we can't decode should be logged, but there's nothing critical here.
                // Processes are free to send us malformed messages over this socket, and we are
                // free to ignore them.
                Err(decode_error) => {
                    self.error_server_addr
                        .do_send(MessageDecodeError(Some(decode_error), bytes));
                }
            },
        };

        // Loop again, sending Recvs until a signal is queued up that kills the actor.
        match ctx.address().try_send(Recv) {
            Err(SendError::Closed(_)) => (),
            _ => ctx.address().do_send(Recv),
        };
    }
}
