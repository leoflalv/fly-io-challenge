use async_trait::async_trait;
use maelstrom::{done, protocol::Message, Node, Result, Runtime};

#[derive(Clone, Default)]
pub struct Handler {}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        if req.get_type() == "echo" {
            let echo = req
                .body
                .clone()
                .with_type("echo_ok")
                .with_reply_to(req.body.msg_id);

            return runtime.reply(req, echo).await;
        }

        done(runtime, req)
    }
}
