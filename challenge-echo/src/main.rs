use std::sync::Arc;

use challenge_echo::handler::Handler;
use maelstrom::{Result, Runtime};

fn main() -> Result<()> {
    Runtime::init(try_main())
}

async fn try_main() -> Result<()> {
    let handler = Arc::new(Handler::default());
    Runtime::new().with_handler(handler).run().await
}
