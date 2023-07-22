1) Go to Discord Developer Portal
2) Create a new Application
3) Under Privileged Gateway Intents turn on Message Content Intent (If needed, Permissions int of 326417590272 may work)
4) Click the Bot section
  * Reset the bot token
  * Make a copy of `example.env` and change the name to `.env`
  * Fill in the value of DISCORD_TOKEN with your bot token
5) Click OAuth2 section create an invite URL and save this. Open it in a browser to add the bot to your server
6) Build and run the binary with `cargo run`
