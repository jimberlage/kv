use actix::{Actor, Addr, Context};
use zmq;

use crate::server::errors::{ErrorServer, SocketConnectionError, SocketOpenError};
use crate::server::set::SetAgent;

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
