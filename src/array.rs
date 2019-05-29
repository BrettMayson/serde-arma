use serde::de::{DeserializeSeed, SeqAccess};

use crate::error::{Error, Result};

pub struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut crate::Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    pub fn new(de: &'a mut crate::Deserializer<'de>) -> Self {
        CommaSeparated {
            de,
            first: true,
        }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.de.peek_char()? == '}' {
            return Ok(None);
        }
        // Comma is required before every element except the first.
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ExpectedArrayComma);
        }
        self.first = false;
        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}
