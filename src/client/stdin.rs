use std::collections::VecDeque;
use std::io::{self, Read};

use actix::prelude::SendError;
use actix::{Actor, Addr, Context, Handler, Message};

use crate::client::errors::{ErrorServer, StdinReadError};
use crate::client::messenger::{MessengerServer, SetInsert};

pub struct StdinReaderServer {
    current_chunk: Vec<u8>,
    current_sep_idx: usize,
    buf: [u8; 5242880],
    chunks: VecDeque<Vec<u8>>,
    error_server_addr: Addr<ErrorServer>,
    messenger_server_addr: Addr<MessengerServer>,
    name: String,
    sep: Vec<u8>,
}

impl StdinReaderServer {
    pub fn new(
        error_server_addr: Addr<ErrorServer>,
        messenger_server_addr: Addr<MessengerServer>,
        name: String,
        sep: Vec<u8>,
    ) -> StdinReaderServer {
        StdinReaderServer {
            current_chunk: vec![],
            current_sep_idx: 0,
            buf: [0; 5242880],
            chunks: VecDeque::new(),
            error_server_addr,
            messenger_server_addr,
            name,
            sep,
        }
    }

    fn parse_chunks(&mut self, bytes_read: usize) {
        if self.sep.len() == 0 {
            for i in 0..bytes_read {
                self.current_chunk.push(self.buf[i]);
            }

            self.buf = [0; 5242880];

            return;
        }

        let last_sep_idx = self.sep.len() - 1;

        for i in 0..bytes_read {
            let datum = self.buf[i];
            if datum != self.sep[self.current_sep_idx] {
                self.current_sep_idx = 0;
                self.current_chunk.push(datum);
            } else if self.current_sep_idx == last_sep_idx {
                self.current_sep_idx = 0;
                if self.current_chunk.len() > 0 {
                    self.chunks.push_back(self.current_chunk.clone());
                    self.current_chunk = vec![];
                }
            } else {
                self.current_sep_idx += 1;
            }
        }

        self.buf = [0; 5242880];
    }

    fn flush_chunks(&mut self) {
        while let Some(chunk) = self.chunks.pop_front() {
            match self.messenger_server_addr.try_send(SetInsert {
                name: self.name.clone(),
                value: chunk,
            }) {
                Err(SendError::Full(SetInsert { name: _, value })) => {
                    self.chunks.push_front(value);
                    break;
                }
                Err(SendError::Closed(SetInsert { name: _, value })) => {
                    self.chunks.push_front(value);
                    break;
                }
                Ok(()) => (),
            }
        }
    }
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
            }
            Err(error) => {
                // TODO: Find a better way to handle this error.
                self.error_server_addr
                    .try_send(StdinReadError(error))
                    .unwrap()
            }
        }
    }
}
