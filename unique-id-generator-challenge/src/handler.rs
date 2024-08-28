use async_trait::async_trait;
use maelstrom::{done, protocol::Message, Node, Result, Runtime};
use ulid::Ulid;

#[derive(Clone, Default)]
pub struct Handler {}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        match req.get_type() {
            "generate" => {
                let ulid = Ulid::new();
                let new_id = ulid.to_string();
                let mut msg = req.body.clone().with_type("generate_ok");
                msg.extra.insert("id".into(), new_id.into());

                runtime.reply(req, msg).await
            }
            _ => done(runtime, req),
        }
    }
}
