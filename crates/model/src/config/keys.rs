//! The `[keys]` table: an action's name and the key or keys it is bound to,
//! as the user wrote them. Nothing here knows what a key is: the screen reads
//! the text, checks it and says what was wrong (`ui/src/keys/`). A value of
//! the wrong kind is kept as `Wrong` instead of failing the whole file, so one
//! typo costs one key and the screen can name it.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};

/// What one entry of `[keys]` says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyValue {
    /// `reply = "r"`
    One(String),
    /// `reply = ["r", "Ctrl+r"]`
    Many(Vec<String>),
    /// Anything else (a number, a table, a list holding a number).
    Wrong,
}

impl KeyValue {
    /// The key texts the entry holds, or `None` for a value of the wrong kind.
    pub fn texts(&self) -> Option<Vec<&str>> {
        match self {
            Self::One(text) => Some(vec![text.as_str()]),
            Self::Many(list) => Some(list.iter().map(String::as_str).collect()),
            Self::Wrong => None,
        }
    }
}

impl Serialize for KeyValue {
    fn serialize<S: Serializer>(&self, out: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One(text) => out.serialize_str(text),
            Self::Many(list) => {
                let mut seq = out.serialize_seq(Some(list.len()))?;
                for text in list {
                    seq.serialize_element(text)?;
                }
                seq.end()
            }
            // A value of the wrong kind has no honest spelling; an empty text
            // is read back as "no key given", which is what it was.
            Self::Wrong => out.serialize_str(""),
        }
    }
}

/// One element of a list: text, or something else.
struct Piece(Option<String>);

impl<'de> Deserialize<'de> for Piece {
    fn deserialize<D: Deserializer<'de>>(input: D) -> Result<Self, D::Error> {
        input.deserialize_any(PieceVisitor)
    }
}

struct PieceVisitor;

impl<'de> Visitor<'de> for PieceVisitor {
    type Value = Piece;

    fn expecting(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str("a key as text")
    }

    fn visit_str<E>(self, text: &str) -> Result<Piece, E> {
        Ok(Piece(Some(text.to_string())))
    }

    fn visit_i64<E>(self, _: i64) -> Result<Piece, E> {
        Ok(Piece(None))
    }

    fn visit_u64<E>(self, _: u64) -> Result<Piece, E> {
        Ok(Piece(None))
    }

    fn visit_f64<E>(self, _: f64) -> Result<Piece, E> {
        Ok(Piece(None))
    }

    fn visit_bool<E>(self, _: bool) -> Result<Piece, E> {
        Ok(Piece(None))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Piece, A::Error> {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(Piece(None))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Piece, A::Error> {
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
        Ok(Piece(None))
    }
}

impl<'de> Deserialize<'de> for KeyValue {
    fn deserialize<D: Deserializer<'de>>(input: D) -> Result<Self, D::Error> {
        input.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = KeyValue;

    fn expecting(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str("a key, or a list of keys")
    }

    fn visit_str<E>(self, text: &str) -> Result<KeyValue, E> {
        Ok(KeyValue::One(text.to_string()))
    }

    fn visit_i64<E>(self, _: i64) -> Result<KeyValue, E> {
        Ok(KeyValue::Wrong)
    }

    fn visit_u64<E>(self, _: u64) -> Result<KeyValue, E> {
        Ok(KeyValue::Wrong)
    }

    fn visit_f64<E>(self, _: f64) -> Result<KeyValue, E> {
        Ok(KeyValue::Wrong)
    }

    fn visit_bool<E>(self, _: bool) -> Result<KeyValue, E> {
        Ok(KeyValue::Wrong)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<KeyValue, A::Error> {
        let mut list = Vec::new();
        let mut wrong = false;
        while let Some(Piece(piece)) = seq.next_element::<Piece>()? {
            match piece {
                Some(text) => list.push(text),
                None => wrong = true,
            }
        }
        Ok(if wrong { KeyValue::Wrong } else { KeyValue::Many(list) })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<KeyValue, A::Error> {
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
        Ok(KeyValue::Wrong)
    }
}

/// The table, by action name. Only what the file says: an action that is not
/// in it keeps its own key.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeysConfig(pub BTreeMap<String, KeyValue>);

impl KeysConfig {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&KeyValue> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &KeyValue)> {
        self.0.iter()
    }

    /// An entry naming one key.
    pub fn with(mut self, name: &str, key: &str) -> Self {
        self.0.insert(name.to_string(), KeyValue::One(key.to_string()));
        self
    }
}
