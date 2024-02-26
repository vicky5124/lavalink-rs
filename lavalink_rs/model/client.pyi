import typing as t
from lavalink_rs import LavalinkClient, GuildId


class NodeDistributionStrategy:
    def new() -> t.Self:
        ...

    def sharded() -> t.Self:
        ...

    def round_robin() -> t.Self:
        ...

    def main_fallback() -> t.Self:
        ...

    def lowest_load() -> t.Self:
        ...

    def highest_free_memory() -> t.Self:
        ...

    def custom(
        func: t.Callable[[LavalinkClient, t.Union[GuildId, int]], t.Awaitable[int]]
    ) -> t.Self:
        ...
