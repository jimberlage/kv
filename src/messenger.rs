use actix::{Actor, Addr, Context, Handler, Message};
use zmq;

use crate::set::SetAgent;

pub struct MessengerServer {
    ctx: zmq::Context,
    host: String,
    port: u16,
    set_agent_addr: Addr<SetAgent>,
    socket: Option<zmq::Socket>,
}

impl MessengerServer {
    pub fn new(host: &str, port: u16, set_agent_addr: Addr<SetAgent>) -> Self {
        MessengerServer {
            ctx: zmq::Context::new(),
            host: host.to_owned(),
            port,
            set_agent_addr,
            socket: None,
        }
    }
}

impl Actor for MessengerServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        match self.ctx.socket(zmq::SocketType::PUB) {
            Ok(socket) => {
                self.socket = Some(socket);
                // TODO: Would prefer not to panic here.
                if let Err(error) = self.socket.as_ref().unwrap().connect(&format!("tcp://{}:{}", self.host, self.port)) {
                    panic!(error);
                }
            },
            Err(error) => panic!(error),
        }
    }
}