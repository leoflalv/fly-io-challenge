use async_trait::async_trait;
use maelstrom::{done, protocol::Message, Node, Result, Runtime};

#[derive(Clone, Default)]
pub struct Handler {}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        match req.get_type() {
            "generate" => {
                unimplemented!()
            }
            _ => done(runtime, req),
        }
    }
}
