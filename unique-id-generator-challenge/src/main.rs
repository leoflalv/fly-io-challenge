use std::sync::Arc;

use maelstrom::{Result, Runtime};
use unique_id_generator_challenge::handler::Handler;

fn main() -> Result<()> {
    Runtime::init(try_main())
}

async fn try_main() -> Result<()> {
    let handler = Arc::new(Handler::default());
    Runtime::new().with_handler(handler).run().await
}
