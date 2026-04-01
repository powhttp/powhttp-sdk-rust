use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::time::Duration;

pub(crate) mod base64_bytes {
    use super::*;

    pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&BASE64_STANDARD.encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        BASE64_STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}

pub(crate) mod hex_2d {
    use super::*;

    pub fn serialize<S: Serializer>(data: &[Vec<u8>], serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(data.len()))?;
        for item in data {
            seq.serialize_element(&hex::encode(item))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error> {
        let strings: Vec<String> = Vec::deserialize(deserializer)?;
        strings
            .iter()
            .map(|s| hex::decode(s).map_err(serde::de::Error::custom))
            .collect()
    }
}

pub(crate) mod option_duration_millis {
    use super::*;

    pub fn serialize<S: Serializer>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error> {
        value.map(|d| d.as_millis() as u64).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Duration>, D::Error> {
        Ok(Option::<u64>::deserialize(deserializer)?.map(Duration::from_millis))
    }
}
