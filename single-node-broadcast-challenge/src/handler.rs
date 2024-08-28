use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use maelstrom::{done, protocol::Message, Node, Result, Runtime};
use serde_json::{json, Value};

#[derive(Clone, Default)]
pub struct Handler {
    store: Arc<Mutex<Vec<Value>>>,
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
            "topology" => runtime.reply_ok(req).await,
            _ => done(runtime, req),
        }
    }
}
