use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity};
mod commands;
mod config;
mod constants;
mod llm;
mod models;
mod utils;
use commands::{chat, greeting};
use utils::event_handler as on_message_handler;

use models::Data;
#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env file
    env_logger::init();

    let options = poise::FrameworkOptions {
        commands: vec![greeting(), chat()],
        event_handler: |ctx, event, framework_ctx, data| {
            Box::pin(on_message_handler(ctx, event, framework_ctx, data))
        },
        ..Default::default()
    };
    let framework = poise::Framework::builder()
        .options(options)
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
