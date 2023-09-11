use crate::config::BOT_NAME;
use crate::utils::bot_instructions;

pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type PoiseContext<'a> = poise::Context<'a, Data, Error>;

pub enum CompletionResult {
    Ok,
    ModerationBlocked,
    ModerationFlagged,
    TooLong,
    InvalidRequest,
    OtherError,
}

pub struct CompletionData {
    pub status: CompletionResult,
    pub reply_text: Option<String>,
    pub status_text: Option<String>,
}
pub struct SimpleMessage {
    pub user: String,
    pub text: String,
}

pub struct Conversation {
    pub messages: Vec<SimpleMessage>,
    pub user: String,
}

impl SimpleMessage {
    pub fn render(&self) -> String {
        format!("{}: {}", self.user, self.text)
    }
}

impl Conversation {
    pub fn render(&self) -> String {
        let mut rendered = String::new();

        // Prepend the bot's instruction
        let bot_message = SimpleMessage {
            user: BOT_NAME.to_string(),
            text: bot_instructions(),
        };
        rendered.push_str(&bot_message.render());

        for message in &self.messages {
            rendered.push_str("\n");
            rendered.push_str(&message.render());
        }
        rendered
    }
}
