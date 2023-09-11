use crate::config::{BOT_INSTRUCTIONS, BOT_NAME};
use crate::constants::{
    ACTIVATE_THREAD_PREFIX, INACTIVATE_THREAD_PREFIX, MAX_CHARS_PER_REPLY_MSG, MAX_THREAD_MESSAGES,
    SECONDS_DELAY_RECEIVING_MSG,
};
use crate::models::{CompletionData, CompletionResult, Conversation, Data, Error, SimpleMessage};
use llm_chain::{executor, parameters, prompt};
use poise::serenity_prelude::{
    self as serenity, Channel, ChannelId, ChannelType, Context, Message, MessageId, UserId,
};
use poise::Event;

pub fn bot_instructions() -> String {
    format!(
        "You are a bot named {}. {} Remember, do not start your response with 'Assistant:' or '{}:'",
        BOT_NAME, BOT_INSTRUCTIONS, BOT_NAME
    )
}

pub fn trim_assistant_prefix(response: &str) -> String {
    let re = regex::Regex::new(&format!("(?:Assistant|{})[^:]*:", BOT_NAME)).unwrap();
    let transformed_message = re.replace(response, "");
    transformed_message.trim().to_string()
}

pub fn discord_message_to_simple_message(message: &Message) -> Option<SimpleMessage> {
    // If the message has embeds, we get the first field value and return it as a Message
    if let Some(embed) = message.embeds.first() {
        if let Some(field) = embed.fields.first() {
            return Some(SimpleMessage {
                user: field.name.clone(),
                text: field.value.clone(),
            });
        }
    }

    // If the message content is not empty, return it as a Message
    if !message.content.is_empty() {
        return Some(SimpleMessage {
            user: message.author.name.clone(),
            text: message.content.clone(),
        });
    }

    None
}

pub fn split_into_shorter_messages(message: &str) -> Vec<String> {
    message
        .chars()
        .collect::<Vec<char>>()
        .chunks(MAX_CHARS_PER_REPLY_MSG)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

pub async fn is_last_message_stale(
    interaction_message: MessageId,
    last_channel: ChannelId,
    last_message: MessageId,
    bot_id: UserId,
    ctx: &Context,
) -> Result<bool, Error> {
    if interaction_message != last_message {
        let last_msg = last_channel.message(&ctx.http, last_message).await?;
        Ok(last_msg.author.id != bot_id)
    } else {
        Ok(false)
    }
}

pub async fn close_thread(ctx: &Context, channel_id: ChannelId) -> serenity::Result<()> {
    let thread = channel_id.to_channel(ctx).await?;

    if let Channel::Guild(mut channel) = thread {
        let current_name = channel.name.clone(); // Get the current name of the thread
        let new_name = format!("{}{}", INACTIVATE_THREAD_PREFIX, current_name); // Prefix with inactive

        channel
            .edit(ctx, |c| {
                c.name(&new_name); // Set the new name
                c
            })
            .await?;

        channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.description("This thread has been closed due to too long message");
                    e.colour((255, 255, 0))
                })
            })
            .await?;
    }

    Ok(())
}

async fn generate_completion_response(
    messages: Vec<SimpleMessage>,
    user: &str,
) -> Result<CompletionData, Error> {
    // Create a Conversation instance
    let conversation = Conversation {
        messages: messages,
        user: user.to_string(),
    };

    // Render the conversation to a single string
    let prompt: String = format!("{}\n{}:", conversation.render(), BOT_NAME);
    log::info!("Conversation to prompt with: {:?}", prompt);

    let exec = executor!()?;

    // Use the prompt with the executor to get the response
    let res = prompt!(prompt).run(&parameters!(), &exec).await?;

    // Return the result
    Ok(CompletionData {
        status: CompletionResult::Ok,
        reply_text: Some(trim_assistant_prefix(&res.to_string())), // Update the response here
        status_text: None,
    })
}

pub async fn process_response(
    ctx: &Context,
    user: String,
    thread: ChannelId,
    response_data: CompletionData,
) {
    let status = response_data.status;
    let reply_text = response_data.reply_text;
    let status_text = response_data.status_text;

    match status {
        CompletionResult::Ok => {
            let mut sent_message: Option<Message> = None;

            match reply_text {
                Some(text) => {
                    if text.is_empty() {
                        sent_message = Some(
                            thread
                                .send_message(&ctx.http, |m| {
                                    m.embed(|e| {
                                        e.description("**Invalid response** - empty response")
                                            .colour((255, 255, 0))
                                    })
                                })
                                .await
                                .unwrap(),
                        );
                    } else {
                        let shorter_response = split_into_shorter_messages(&text);
                        for r in shorter_response {
                            sent_message = Some(
                                thread
                                    .send_message(&ctx.http, |m| m.content(r))
                                    .await
                                    .unwrap(),
                            );
                        }
                    }
                }
                None => {
                    sent_message = Some(
                        thread
                            .send_message(&ctx.http, |m| {
                                m.embed(|e| {
                                    e.description("**Invalid response** - empty response")
                                        .colour((255, 255, 0))
                                })
                            })
                            .await
                            .unwrap(),
                    );
                }
            }
        }
        CompletionResult::TooLong => {
            close_thread(ctx, thread).await.unwrap();
        }
        CompletionResult::InvalidRequest => {
            thread
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(format!(
                            "**Invalid request** - {}",
                            status_text.unwrap_or_else(|| String::from(""))
                        ))
                        .colour((255, 255, 0))
                    })
                })
                .await
                .unwrap();
        }
        _ => {
            thread
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(format!(
                            "**Error** - {}",
                            status_text.unwrap_or_else(|| String::from(""))
                        ))
                        .colour((255, 255, 0))
                    })
                })
                .await
                .unwrap();
        }
    }
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    framework_ctx: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    if let Event::Message { new_message } = event {
        let msg: &Message = new_message;
        log::info!("Received a new message: {:?}", msg.content);

        // ignore messages from the bot
        if msg.author.id == ctx.cache.current_user_id() {
            return Ok(());
        }

        // ignore messages not in a thread
        let channel = match msg.channel_id.to_channel(&ctx.http).await {
            Ok(channel) => channel,
            Err(_) => return Ok(()),
        };

        let thread = if let Channel::Guild(channel) = channel {
            if let ChannelType::PublicThread | ChannelType::PrivateThread = channel.kind {
                channel
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        // ignore threads not created by the bot
        if thread.owner_id.unwrap() != ctx.cache.current_user_id() {
            return Ok(());
        }

        // ignore threads that are archived locked or title is not what we want
        // if thread.archived || thread.locked || !thread.name.starts_with(ACTIVATE_THREAD_PREFIX) {
        if thread.name.starts_with(ACTIVATE_THREAD_PREFIX) {
            // ignore this thread
            return Ok(());
        }

        if u32::from(thread.message_count.unwrap()) > MAX_THREAD_MESSAGES {
            // too many messages, no longer going to reply
            close_thread(&ctx, thread.id).await.unwrap();
            return Ok(());
        }

        // wait a bit in case user has more messages
        tokio::time::sleep(std::time::Duration::from_secs(SECONDS_DELAY_RECEIVING_MSG)).await;
        let last_message = thread.last_message_id.unwrap();
        if is_last_message_stale(
            msg.id,
            thread.id,
            last_message,
            ctx.cache.current_user_id(),
            &ctx,
        )
        .await
        .unwrap()
        {
            // there is another message, so ignore this one
            return Ok(());
        }
        let mut channel_messages = thread
            .messages(&ctx.http, |m| m)
            .await
            .unwrap()
            .into_iter()
            .filter_map(|m| discord_message_to_simple_message(&m))
            .collect::<Vec<_>>();
        channel_messages.reverse();

        // generate the response
        thread.clone().start_typing(&ctx.http).unwrap();
        let response_data = generate_completion_response(channel_messages, &msg.author.name)
            .await
            .unwrap();

        // Check if the last message is stale again before sending a response
        if is_last_message_stale(
            msg.id,
            thread.id,
            last_message,
            ctx.cache.current_user_id(),
            &ctx,
        )
        .await
        .unwrap()
        {
            // there is another message and its not from us, so ignore this response
            return Ok(());
        }
        // send response
        process_response(&ctx, msg.author.name.clone(), thread.id, response_data).await;
        Ok(())
    } else {
        Ok(())
    }
}
