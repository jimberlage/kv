use std::io;

use actix::{Actor, Context, Handler, Message};
use log::error;
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
pub struct SocketSendError {
    pub error: zmq::Error,
    pub host: String,
    pub port: u16,
}

impl Handler<SocketSendError> for ErrorServer {
    type Result = ();

    fn handle(
        &mut self,
        SocketSendError { error, host, port }: SocketSendError,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        error!(
            "Could not send a message over the ZeroMQ socket at tcp://{}:{}; got error: {}",
            host, port, error
        )
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StdinReadError(pub io::Error);

impl Handler<StdinReadError> for ErrorServer {
    type Result = ();

    fn handle(
        &mut self,
        StdinReadError(error): StdinReadError,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        error!("Could not read from stdin; got error: {}", error)
    }
}
