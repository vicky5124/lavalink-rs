import typing as t

import lightbulb

from bot import Bot


class Context(lightbulb.Context):
    @property
    def bot(self) -> Bot:
        return t.cast(Bot, self.app)


class Plugin(lightbulb.Plugin):
    @property
    def bot(self) -> Bot:
        return t.cast(Bot, self.app)
