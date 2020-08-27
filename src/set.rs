use std::collections::HashSet;

use actix::{Actor, Context};

pub struct SetServer {
    data: HashSet<String>,
}

impl SetServer {
    pub fn new() -> SetServer {
        SetServer { data: HashSet::new() }
    }
}

impl Actor for SetServer {
    type Context = Context<Self>;
}