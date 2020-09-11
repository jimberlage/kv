use std::collections::{HashMap, HashSet};

use actix::{Actor, Context, Handler, Message};

pub struct SetAgent {
    data: HashMap<String, HashSet<Vec<u8>>>,
}

impl SetAgent {
    pub fn new() -> SetAgent {
        SetAgent {
            data: HashMap::new(),
        }
    }
}

impl Actor for SetAgent {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Insert {
    pub id: u32,
    pub name: String,
    pub value: Vec<u8>,
}

impl Handler<Insert> for SetAgent {
    type Result = bool;

    fn handle(
        &mut self,
        Insert { id: _, name, value }: Insert,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        match self.data.get_mut(&name) {
            None => {
                let mut inner = HashSet::new();
                inner.insert(value);
                let _ = self.data.insert(name, inner);
                true
            }
            Some(inner) => inner.insert(value),
        }
    }
}
