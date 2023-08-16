use std::fmt::Display;
use std::num::ParseIntError;
use std::pin::Pin;
use std::{future::Future, str::FromStr};

use serde::{de, Deserialize, Deserializer};

pub mod events;
pub mod http;
pub mod player;
pub mod track;
pub mod search;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UserId(pub u64);
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GuildId(pub u64);

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
    GuildId as SerenityGuildId,
    UserId as SerenityUserId,
};

#[cfg(feature = "serenity")]
impl From<SerenityUserId> for UserId {
    fn from(id: SerenityUserId) -> UserId {
        UserId(id.0)
    }
}

#[cfg(feature = "serenity")]
impl From<SerenityGuildId> for GuildId {
    fn from(id: SerenityGuildId) -> GuildId {
        GuildId(id.0)
    }
}

#[cfg(feature = "twilight")]
use twilight_model::id::{
    Id, marker::{GuildMarker, UserMarker}
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

#[cfg(feature = "songbird")]
use songbird_dep::id::{
    GuildId as SongbirdGuildId,
    UserId as SongbirdUserId,
};

#[cfg(feature = "songbird")]
impl From<SongbirdUserId> for UserId {
    fn from(id: SongbirdUserId) -> UserId {
        UserId(id.0)
    }
}

#[cfg(feature = "songbird")]
impl From<SongbirdGuildId> for GuildId {
    fn from(id: SongbirdGuildId) -> GuildId {
        GuildId(id.0)
    }
}
