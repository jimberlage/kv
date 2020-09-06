use std::io::{self, Read};

use actix::{Actor, Addr, Context, Handler, Message};

use crate::client::messenger::MessengerServer;
use crate::client::errors::{ErrorServer, StdinReadError};

pub struct StdinReaderServer {
    buf: [u8; 5242880],
    chunks: Vec<Vec<u8>>,
    error_server_addr: Addr<ErrorServer>,
    messenger_server_addr: Addr<MessengerServer>,
    sep: Vec<u8>,
}

impl StdinReaderServer {
    pub fn new(error_server_addr: Addr<ErrorServer>, messenger_server_addr: Addr<MessengerServer>, sep: Vec<u8>) -> StdinReaderServer {
        StdinReaderServer {
            buf: [0; 5242880],
            chunks: vec![],
            error_server_addr,
            messenger_server_addr,
            sep,
        }
    }

    fn parse_chunks(&mut self, bytes_read: usize) {
        if self.sep.len() == 0 {
            return
        }

        let mut current_chunk = vec![];
        let mut current_sep_idx = 0;
        let last_sep_idx = self.sep.len() - 1;

        for i in 0..bytes_read {
            let datum = self.buf[i];
            if datum != self.sep[current_sep_idx] {
                current_sep_idx = 0;
                current_chunk.push(datum);
            } else if current_sep_idx == last_sep_idx {
                current_sep_idx = 0;
                if current_chunk.len() > 0 {
                    self.chunks.push(current_chunk);
                    current_chunk = vec![];
                }
            } else {
                current_sep_idx += 1;
            }
        }
    }

    fn flush_chunks(&mut self) {}
}

impl Actor for StdinReaderServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessChunks;

impl Handler<ProcessChunks> for StdinReaderServer {
    type Result = ();

    fn handle(&mut self, _: ProcessChunks, _ctx: &mut Context<Self>) -> Self::Result {
        match io::stdin().lock().read(&mut self.buf) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    self.parse_chunks(bytes_read);
                }
                self.flush_chunks();
            },
            Err(error) => {
                self.error_server_addr.try_send(StdinReadError(error)).unwrap()
            },
        }
    }
}
