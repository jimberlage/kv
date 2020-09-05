use actix::{Actor, Addr, Context, Handler, Message, AsyncContext};
use zmq;

use crate::errors::{ErrorServer, SocketConnectionError, SocketOpenError};
use crate::set::SetAgent;

pub struct MessengerServer {
    ctx: zmq::Context,
    host: String,
    port: u16,
    error_server_addr: Addr<ErrorServer>,
    set_agent_addr: Addr<SetAgent>,
    socket: Option<zmq::Socket>,
}

impl MessengerServer {
    pub fn new(host: &str, port: u16, error_server_addr: Addr<ErrorServer>, set_agent_addr: Addr<SetAgent>) -> Self {
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

    fn started(&mut self, ctx: &mut Self::Context) {
        match self.ctx.socket(zmq::SocketType::PUB) {
            Err(error) => self.error_server_addr.do_send(SocketOpenError(error)),
            Ok(socket) => {
                self.socket = Some(socket);
                if let Err(error) = self.socket.as_ref().unwrap().connect(&format!("tcp://{}:{}", self.host, self.port)) {
                    self.error_server_addr.do_send(SocketConnectionError {
                        error,
                        host: self.host.clone(),
                        port: self.port,
                    })
                }
            },
        }

        // ctx.address().do_send(Tick);
    }
}

// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct Tick;
//
// impl Handler<Tick> for MessengerServer {
//     type Result = ();
//
//     fn handle(&mut self, _: Tick, ctx: &mut Context<MessengerServer>) {
//         ctx.address().do_send(Tick);
//     }
// }
