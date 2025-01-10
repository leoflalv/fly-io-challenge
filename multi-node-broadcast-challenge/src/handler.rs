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

impl Handler {
    fn get_neighbors_to_visit(&self, node_id: &str, path: &[String]) -> Vec<String> {
        let neighbors = self.topology.lock().unwrap().get(node_id).unwrap().clone();

        let neighbors_to_visit: Vec<String> = neighbors
            .iter()
            .filter(|n| !path.contains(n))
            .cloned()
            .collect();

        neighbors_to_visit
    }

    pub fn parse_to_graph(&self, value: &Value) -> Result<()> {
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

        *self.topology.lock().unwrap() = graph;
        Ok(())
    }

    async fn handle_broadcast(&self, runtime: Runtime, req: Message) -> Result<()> {
        let store = self.store.clone();
        let message = req
            .body
            .extra
            .get("message")
            .unwrap_or_else(|| {
                log::error!("Message is missing in broadcast request");
                &Value::Null
            })
            .clone();
        let message_arc = Arc::new(message);

        let mut path: Vec<String> = req
            .body
            .extra
            .get("path")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|item| item.as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default();
        path.push(runtime.node_id().to_string());
        let path_arc = Arc::new(path);

        store.lock().unwrap().push((*message_arc).clone());

        let neighbors_to_visit = self.get_neighbors_to_visit(runtime.node_id(), &path_arc);

        for neighbor in neighbors_to_visit {
            let runtime = runtime.clone(); // Ensure Runtime is clonable and Send
            let message = message_arc.clone();
            let path = path_arc.clone();

            tokio::spawn(async move {
                let mut request = Message::default().body.with_type("broadcast");
                request.extra.insert("message".into(), (*message).clone());
                request.extra.insert("path".into(), json!(*path));

                let _ = runtime.rpc(neighbor, request).await;
            });
        }

        runtime.reply_ok(req).await
    }

    async fn handle_read(&self, runtime: Runtime, req: Message) -> Result<()> {
        let store = self.store.clone();
        let mut resp = Message::default().body.with_type("read_ok");
        {
            let data = store.lock().unwrap();
            let array = json!(*data);
            resp.extra.insert("messages".into(), array);
        }

        runtime.reply(req, resp).await
    }

    async fn handle_topology(&self, runtime: Runtime, req: Message) -> Result<()> {
        let incoming_topology = req.body.extra.get("topology").unwrap().clone();
        self.parse_to_graph(&incoming_topology)?;

        runtime.reply_ok(req).await
    }
}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        match req.get_type() {
            "broadcast" => self.handle_broadcast(runtime, req).await,
            "read" => self.handle_read(runtime, req).await,
            "topology" => self.handle_topology(runtime, req).await,
            _ => done(runtime, req),
        }
    }
}
