import os
import hikari
import lightbulb
import lavalink_rs


class Bot(lightbulb.BotApp):
    __slots__ = "lavalink"

    def __init__(self) -> None:
        super().__init__(
            token=os.environ["DISCORD_TOKEN"],
            prefix=",",
            intents=hikari.Intents.ALL_MESSAGES
            | hikari.Intents.GUILDS
            | hikari.Intents.MESSAGE_CONTENT
            | hikari.Intents.GUILD_VOICE_STATES
            | hikari.Intents.GUILD_MEMBERS,
        )

        self.lavalink: lavalink_rs.LavalinkClient

        self.load_extensions_from("./plugins")


if __name__ == "__main__":
    bot = Bot()

    bot.run(
        activity=hikari.Activity(
            name="your weird music taste...",
            type=hikari.ActivityType.WATCHING,
        ),
    )
