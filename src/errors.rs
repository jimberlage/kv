use actix::{Actor, Addr, Context, Handler, Message};
use log::error;
use simple_logger::SimpleLogger;
use zmq;

pub struct ErrorServer {
    engine: SimpleLogger,
}

impl ErrorServer {
    pub fn new() -> ErrorServer { ErrorServer { engine: SimpleLogger::new() } }
}

impl Actor for ErrorServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SocketOpenError(pub zmq::Error);

impl Handler<SocketOpenError> for ErrorServer {
    type Result = ();

    fn handle(&mut self, SocketOpenError(zmq_error): SocketOpenError, _ctx: &mut Context<Self>) -> Self::Result {
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

    fn handle(&mut self, SocketConnectionError { error, host, port }: SocketConnectionError, _ctx: &mut Context<Self>) -> Self::Result {
        error!("Could not connect the ZeroMQ socket over tcp://{}:{}; got error: {}", host, port, error)
    }
}
