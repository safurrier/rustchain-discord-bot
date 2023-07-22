use dotenv::dotenv;
use llm_chain::{executor, parameters, prompt};
use poise::serenity_prelude as serenity;
struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(prefix_command, slash_command)]
async fn greeting(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user: &serenity::User = user.as_ref().unwrap_or_else(|| ctx.author());

    // Create a new ChatGPT executor
    let exec = executor!()?;

    let greeting_prompt = format!(
        "You are a robot assistant for making personalized greetings. \
        Make a personalized greeting for {}",
        user.name
    );

    // Create our prompt...
    let res = prompt!(greeting_prompt)
        .run(&parameters!(), &exec) // ...and run it
        .await?;
    ctx.say(res.to_string()).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![greeting()],
            ..Default::default()
        })
        .intents(serenity::GatewayIntents::MESSAGE_CONTENT)
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
