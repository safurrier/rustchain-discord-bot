use crate::models::{Conversation, Error, PoiseContext, SimpleMessage};
use llm_chain::{executor, parameters, prompt};
use poise::serenity_prelude::{
    self as serenity,
    json::{hashmap_to_json_map, json, JsonMap, Value},
    Colour,
};

use crate::config::BOT_NAME;
use crate::llm::options;
use crate::utils::trim_assistant_prefix;

/// Make a personalized greeting for a user
#[poise::command(prefix_command, slash_command)]
pub async fn greeting(
    ctx: PoiseContext<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user: &serenity::User = user.as_ref().unwrap_or_else(|| ctx.author());

    // use options from llm.rs
    let options = options();

    let exec = executor!(chatgpt, options.clone())?;

    let greeting_prompt = format!(
        "You are a robot assistant for making personalized greetings. \
        Make a personalized greeting for {}",
        user.name
    );

    // Prompt and execute
    let res = prompt!(greeting_prompt)
        .run(&parameters!(), &exec) // ...and run it
        .await?;
    ctx.say(trim_assistant_prefix(&res.to_string())).await?;
    Ok(())
}

/// An LLM chat
#[poise::command(prefix_command, slash_command)]
pub async fn chat(
    ctx: PoiseContext<'_>,
    #[description = "Prompt for the chatbot"] user_prompt: String,
) -> Result<(), Error> {
    // let name = "Let's chat!";
    let http = ctx.serenity_context().http.clone();
    let channel_id = ctx.channel_id().0;

    let user_message = SimpleMessage {
        user: ctx.author().name.clone(),
        text: user_prompt.clone(),
    };

    // Construct the conversation
    let conversation = Conversation {
        messages: vec![user_message],
        user: ctx.author().name.clone(), // This might not be necessary for your render function but I'm including it just in case.
    };

    // Render the conversation to get the new prompt
    // Bot instructions will be injected at the beginning of the conversation
    // let prompt = conversation.render();
    let prompt: String = format!("{}\n{}:", conversation.render(), BOT_NAME);

    // Create the embed message.
    let message = poise::send_reply(ctx, |m| {
        m.embed(|e| {
            e.title(format!("{} initiated a chat", ctx.author().name));
            e.description(&user_prompt[..std::cmp::min(user_prompt.len(), 1000)]); // Limit the description to 1000 characters
            e.color(Colour::from_rgb(85, 57, 204)); // Blurple
            e
        })
    })
    .await?;

    let message_id = message.message().await?.id.0;

    // Construct thread name
    const LIMIT: usize = 20;
    // Get the first sentence of the user's input message
    let mut name = user_prompt
        .split('.')
        .next()
        .unwrap_or("Let's chat!")
        .to_owned();
    // If the name is longer than the limit, truncate it and add an ellipsis
    if name.len() > LIMIT {
        name.truncate(LIMIT);
        name.push_str("...");
    }

    let map = hashmap_to_json_map::<std::collections::hash_map::RandomState, _>(
        std::collections::HashMap::from_iter([("name".to_owned(), Value::String(name.to_owned()))]),
    );

    let options = options();
    let exec = executor!(chatgpt, options.clone())?;

    // Use the user's prompt
    let res = prompt!(prompt).run(&parameters!(), &exec).await?;

    let mut thread_map = JsonMap::new();
    thread_map.insert("content".to_string(), json!(res.to_string()));
    thread_map.insert("username".to_string(), json!("Bot Name"));
    tokio::spawn(async move {
        // Get the GuildChannel struct from creating a public thread
        let guild_channel = http
            .create_public_thread(channel_id, message_id, &map)
            .await
            .expect("Failed to create public thread");

        // Get the thread id from the GuildChannel struct
        let thread_id = guild_channel.id;

        // Send a message in the thread using the thread id
        thread_id
            .send_message(http, |m| {
                // Set the content of the message to the chatbot response
                m.content(trim_assistant_prefix(&res.to_string()));
                // Optionally, you can also set other properties of the message, such as an embed or attachments
                m
            })
            .await
            .expect("Failed to send message in thread");
    });

    Ok(())
}
