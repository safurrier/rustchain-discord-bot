# rustchain-discord-bot

`rustchain-discord-bot` is a powerful Discord bot designed to interact with users in real-time chat within a Discord thread using the `/chat` command. By default, the bot integrates with OpenAI to leverage various GPT models for dynamic and rich conversations. The underlying code is based on the Rust crate [`llm-chain`](https://github.com/sobelio/llm-chain), which is inspired by LangChain. This provides the bot with extensibility and customization, offering a wide array of potential capabilities.

## Features

- Real-time chat interaction using the `/chat` command.
- Integration with OpenAI for dynamic conversation abilities.
- Customizable bot name and OpenAI model configurations.
- Extensible and customizable codebase thanks to `llm-chain`.

## Setup

### 1. Discord Developer Portal Configuration

- **Step 1**: Navigate to [Discord Developer Portal](https://discord.com/developers/applications).

- **Step 2**: Click on the `New Application` button to create a new application.

- **Step 3**: Click on the `Bot` section on the left sidebar.. Under the `Privileged Gateway Intents` section, ensure the `Message Content Intent` is enabled. Alternatively, you may use the Permissions intent with a value of `326417590272`.

- **Step 4**: Reset bot token.
  - Still on the `Bot` section
  - **Reset** the bot token.
  - **Create a copy** of `example.env` and rename it to `.env`.
  - Fill in the value of `DISCORD_TOKEN` with your bot token.
  - Ensure your bot's name here matches the bot name specified in the `config.rs` under the variable `BOT_NAME`.

- **Step 5**: Invite the bot to your server
  - In the `OAuth2` section, subsection `URL Generator`, create an invite URL and save it.
    - For bot permissinos, replicate those from step 3 (or at minimum `Send Messages`, `Create Public Threads`, `Send Messages in Threads`, `Manage Messages`, `Read Message History`, `Use Slash Commands`)
  - Open this URL in a browser to add the bot to your server.

### 2. Bot Configuration

- Navigate to the `config.rs` file to set your bot name and OpenAI model.
- Additional LLM settings, such as temperature, can be added in `llm.rs` under the `options` function.

### 3. Running the Bot

- Build and run the bot using the command: `cargo run`.
