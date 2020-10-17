use actix::{Actor, Context, Handler, Message};
use log::error;
use prost;
use simple_logger::SimpleLogger;
use zmq;

pub struct ErrorServer {
    engine: SimpleLogger,
}

impl ErrorServer {
    pub fn new() -> ErrorServer {
        ErrorServer {
            engine: SimpleLogger::new(),
        }
    }
}

impl Actor for ErrorServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MessageDecodeError(pub Option<prost::DecodeError>, pub Vec<u8>);

impl Handler<MessageDecodeError> for ErrorServer {
    type Result = ();

    fn handle(
        &mut self,
        MessageDecodeError(error, message): MessageDecodeError,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        match error {
            Some(error) => {
                error!(
                    "Could not decode a message sent by a client; got this error with this message (base64-encoded): {}, {}",
                    error,
                    base64::encode(&message)
                )
            },
            None => {
                error!(
                    "Could not decode a message sent by a client; got message (base64-encoded): {}",
                    base64::encode(&message)
                )
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SocketOpenError(pub zmq::Error);

impl Handler<SocketOpenError> for ErrorServer {
    type Result = ();

    fn handle(
        &mut self,
        SocketOpenError(zmq_error): SocketOpenError,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        error!("Could not open a ZeroMQ socket: {}", zmq_error)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SocketConnectionError {
    pub error: zmq::Error,
    pub host: String,
    pub port: u16,
}

impl Handler<SocketConnectionError> for ErrorServer {
    type Result = ();

    fn handle(
        &mut self,
        SocketConnectionError { error, host, port }: SocketConnectionError,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        error!(
            "Could not connect the ZeroMQ socket over tcp://{}:{}; got error: {}",
            host, port, error
        )
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SocketRecvError {
    pub error: zmq::Error,
    pub host: String,
    pub port: u16,
}

impl Handler<SocketRecvError> for ErrorServer {
    type Result = ();

    fn handle(
        &mut self,
        SocketRecvError { error, host, port }: SocketRecvError,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        error!(
            "Could not retreive data from the ZeroMQ socket on tcp://{}:{}; got error: {}",
            host, port, error
        )
    }
}

// Represents an inability to send a response to the given client, probably causing a timeout on
// their end.
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnsentResponseError {
    pub client_id: u32,
    pub host: String,
    pub port: u16,
}

impl Handler<UnsentResponseError> for ErrorServer {
    type Result = ();

    fn handle(
        &mut self,
        UnsentResponseError { client_id, host, port }: UnsentResponseError,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        error!(
            "Could not send data on the ZeroMQ socket on tcp://{}:{} for client {}",
            host, port, client_id
        )
    }
}
