use std::fmt::Display;
use std::num::ParseIntError;
use std::pin::Pin;
use std::{future::Future, str::FromStr};

use serde::{de, Deserialize, Deserializer};

pub mod events;
pub mod http;
pub mod player;
pub mod track;

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
