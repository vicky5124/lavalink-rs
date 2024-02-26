use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

pub use futures::future::BoxFuture;
use serde::{de, Deserialize, Deserializer};

/// Models related to the lavalink client.
pub mod client;
/// Models related to the lavalink events.
pub mod events;
/// Models related to the lavalink REST API.
pub mod http;
/// Models related to the lavalink Player.
pub mod player;
/// Models related to search engines.
pub mod search;
/// Models related to the tracks.
pub mod track;

#[derive(Clone, Default)]
pub(crate) struct Secret(pub(crate) Box<str>);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<hidden>")
    }
}

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Default, Serialize, Deserialize,
)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
/// A discord User ID.
pub struct UserId(pub u64);
#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Default, Serialize, Deserialize,
)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
/// A discord Guild ID.
pub struct GuildId(pub u64);
#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Default, Serialize, Deserialize,
)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
/// A discord Channel ID.
pub struct ChannelId(pub u64);

impl FromStr for UserId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str(s).map(Self)
    }
}

impl FromStr for GuildId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str(s).map(Self)
    }
}

impl FromStr for ChannelId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str(s).map(Self)
    }
}

impl From<u64> for UserId {
    fn from(i: u64) -> Self {
        Self(i)
    }
}

impl From<u64> for GuildId {
    fn from(i: u64) -> Self {
        Self(i)
    }
}

impl From<u64> for ChannelId {
    fn from(i: u64) -> Self {
        Self(i)
    }
}

pub(crate) fn deserialize_option_number<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let n = i32::deserialize(deserializer)?;
    Ok(match n.cmp(&-1) {
        std::cmp::Ordering::Less => return Err(de::Error::custom("integer {n} is below -1")),
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(n.try_into().unwrap()),
    })
}

pub(crate) fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}

#[cfg(feature = "serenity")]
use serenity_dep::model::id::{
    ChannelId as SerenityChannelId, GuildId as SerenityGuildId, UserId as SerenityUserId,
};

#[cfg(feature = "serenity")]
impl From<SerenityUserId> for UserId {
    fn from(id: SerenityUserId) -> UserId {
        UserId(id.get().into())
    }
}

#[cfg(feature = "serenity")]
impl From<SerenityGuildId> for GuildId {
    fn from(id: SerenityGuildId) -> GuildId {
        GuildId(id.get().into())
    }
}

#[cfg(feature = "serenity")]
impl From<SerenityChannelId> for ChannelId {
    fn from(id: SerenityChannelId) -> ChannelId {
        ChannelId(id.get().into())
    }
}

#[cfg(feature = "twilight")]
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

#[cfg(feature = "twilight")]
impl From<Id<UserMarker>> for UserId {
    fn from(id: Id<UserMarker>) -> UserId {
        UserId(id.get())
    }
}

#[cfg(feature = "twilight")]
impl From<Id<GuildMarker>> for GuildId {
    fn from(id: Id<GuildMarker>) -> GuildId {
        GuildId(id.get())
    }
}

#[cfg(feature = "twilight")]
impl From<Id<ChannelMarker>> for ChannelId {
    fn from(id: Id<ChannelMarker>) -> ChannelId {
        ChannelId(id.get())
    }
}

#[cfg(feature = "twilight16")]
use twilight_model_16::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

#[cfg(feature = "twilight16")]
impl From<Id<UserMarker>> for UserId {
    fn from(id: Id<UserMarker>) -> UserId {
        UserId(id.get())
    }
}

#[cfg(feature = "twilight16")]
impl From<Id<GuildMarker>> for GuildId {
    fn from(id: Id<GuildMarker>) -> GuildId {
        GuildId(id.get())
    }
}

#[cfg(feature = "twilight16")]
impl From<Id<ChannelMarker>> for ChannelId {
    fn from(id: Id<ChannelMarker>) -> ChannelId {
        ChannelId(id.get())
    }
}

#[cfg(feature = "songbird")]
use songbird_dep::id::{
    ChannelId as SongbirdChannelId, GuildId as SongbirdGuildId, UserId as SongbirdUserId,
};

#[cfg(feature = "songbird")]
impl From<SongbirdUserId> for UserId {
    fn from(id: SongbirdUserId) -> UserId {
        UserId(id.0.into())
    }
}

#[cfg(feature = "songbird")]
impl From<SongbirdGuildId> for GuildId {
    fn from(id: SongbirdGuildId) -> GuildId {
        GuildId(id.0.into())
    }
}

#[cfg(feature = "songbird")]
impl From<SongbirdChannelId> for ChannelId {
    fn from(id: SongbirdChannelId) -> ChannelId {
        ChannelId(id.0.into())
    }
}
