use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use maelstrom::{done, protocol::Message, Node, Result, Runtime};
use serde_json::{json, Value};

type Graph = HashMap<String, Vec<String>>;

#[derive(Clone, Default)]
pub struct Handler {
    store: Arc<Mutex<Vec<Value>>>,
    topology: Arc<Mutex<Graph>>,
}

fn parse_to_graph(value: &Value) -> Graph {
    let obj = value.as_object().unwrap();
    let mut graph = Graph::new();

    for (key, val) in obj.iter() {
        let edges = {
            let mut strings = Vec::new();
            let edges = val.as_array().unwrap();
            for v in edges {
                strings.push(v.as_str().unwrap().to_string());
            }
            strings
        };

        graph.insert(key.clone(), edges);
    }

    graph
}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        match req.get_type() {
            "broadcast" => {
                let store = self.store.clone();
                let message = req.body.extra.get("message").unwrap().clone();

                {
                    let mut data = store.lock().unwrap();
                    data.push(message);
                }

                runtime.reply_ok(req).await
            }
            "read" => {
                let store = self.store.clone();
                let mut resp = Message::default().body.with_type("read_ok");
                {
                    let data = store.lock().unwrap();
                    let array = json!(*data);
                    resp.extra.insert("messages".into(), array);
                }

                runtime.reply(req, resp).await
            }
            "topology" => {
                let incoming_topology = req.body.extra.get("topology").unwrap().clone();
                {
                    let mut topology = self.topology.lock().unwrap();
                    *topology = parse_to_graph(&incoming_topology);
                    log::error!("{:?}", topology);
                }

                runtime.reply_ok(req).await
            }
            _ => done(runtime, req),
        }
    }
}
