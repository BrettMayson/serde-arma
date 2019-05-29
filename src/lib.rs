use serde::Deserialize;
use serde::de::{self, Visitor};

#[macro_use]
extern crate lazy_static;

use std::ops::{AddAssign, MulAssign, Neg};

mod array;
mod class;
mod error;

use crate::array::CommaSeparated;
use crate::class::ArmaClass;
use crate::error::{Error, Result};

lazy_static! {
    static ref WHITESPACE: String = String::from(" \r\n\t");
    static ref DIGIT_END: String = String::from(";,} \r\n\t");
}

pub struct Deserializer<'de> {
    input: &'de str,
    next_is_class: bool,
    next_is_key: bool,
    first_reader: bool,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            input,
            next_is_class: false,
            next_is_key: false,
            first_reader: false,
        }
    }
    pub fn from_reader<R: std::io::Read>(mut reader: R) -> Self {
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();
        let sstr: &'static str = Box::leak(text.into_boxed_str());
        Deserializer {
            input: sstr,
            next_is_class: false,
            next_is_key: false,
            first_reader: true,
        }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

pub fn from_reader<'a, R>(reader: R) -> Deserializer<'a>
where
    R: std::io::Read,
{
    Deserializer::from_reader(reader)
}

impl<'de> Deserializer<'de> {
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + std::str::FromStr + std::fmt::Debug,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        let mut s = String::new();
        loop {
            if DIGIT_END.contains(self.peek_char()?) {
                return Ok(s.parse::<T>().unwrap());
            } else {
                s.push(self.next_char()?);
            }
        }
    }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + std::str::FromStr + std::fmt::Debug,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        let mut s = String::new();
        loop {
            if DIGIT_END.contains(self.peek_char()?) {
                return Ok(s.parse::<T>().unwrap());
            } else {
                s.push(self.next_char()?);
            }
        }
    }

    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + std::str::FromStr + std::fmt::Debug,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        let mut s = String::new();
        loop {
            if DIGIT_END.contains(self.peek_char()?) {
                return Ok(s.parse::<T>().unwrap());
            } else {
                s.push(self.next_char()?);
            }
        }
    }

    fn parse_bool(&mut self) -> Result<bool> {
        if self.input.starts_with("true") {
            self.input = &self.input["true".len()..];
            Ok(true)
        } else if self.input.starts_with("false") {
            self.input = &self.input["false".len()..];
            Ok(false)
        } else {
            Err(Error::ExpectedBoolean)
        }
    }

    fn parse_string(&mut self) -> Result<&'de str> {
        if self.next_is_class {
            let spc_pos = self.input.find(' ').unwrap_or(1000);
            let nl_pos = self.input.find('\n').unwrap_or(1000);
            let br_pos = self.input.find('{').unwrap_or(1000);
            let f_pos = if spc_pos < nl_pos { spc_pos } else { nl_pos };
            let mut pos = Some(if f_pos < br_pos { f_pos } else {br_pos});
            if pos == Some(1000) {
                pos = None
            }
            match pos {
                Some(len) => {
                    let s = &self.input[..len].trim();
                    self.input = &self.input[len..];
                    Ok(s)
                }
                None => Err(Error::Eof),
            }
        } else if self.peek_char()? == '"' {
            self.next_char()?;
            let mut s = String::new();
            loop {
                let c = self.next_char()?;
                if c == '"' {
                    if self.peek_char()? == '"' {
                        self.next_char()?;
                        s.push('"');
                    } else {
                        if self.input.starts_with(" \\n \"") {
                            self.input = &self.input[" \\n \"".len()..];
                            s.push('\n');
                        } else {
                            break;
                        }
                    }
                } else {
                    s.push(c);
                }
            }
            let sstr: &'static str = Box::leak(s.into_boxed_str());
            Ok(sstr)
        } else {
            match self.input.find('=') {
                Some(len) => {
                    let s = &self.input[..len].trim();
                    self.input = &self.input[len..];
                    if let Some(pos) = s.find('[') {
                        return Ok(&s[..pos]);
                    }
                    Ok(s)
                }
                None => Err(Error::Eof),
            }
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.first_reader {
            self.first_reader = false;
            self.deserialize_struct("", &[], visitor)
        } else if self.next_is_key {
            self.next_is_key = false;
            self.deserialize_str(visitor)
        } else {
            match self.peek_char()? {
                'n' => self.deserialize_unit(visitor),
                't' | 'f' => self.deserialize_bool(visitor),
                '"' => self.deserialize_str(visitor),
                '0'...'9' | '-' => {
                    let mut s = String::new();
                    let mut i = 0;
                    loop {
                        if DIGIT_END.contains(self.input.chars().nth(i).unwrap()) {
                            if s.contains('.') {
                                return self.deserialize_f64(visitor);
                            } else if s.contains('-') {
                                return self.deserialize_i64(visitor);
                            } else {
                                return self.deserialize_u64(visitor);
                            }
                        } else {
                            s.push(self.input.chars().nth(i).unwrap());
                            i += 1;
                        }
                    }
                },
                '{' => {
                    if self.next_is_class {
                        self.next_is_class = false;
                        self.deserialize_map(visitor)
                    } else {
                        self.deserialize_seq(visitor)
                    }
                },
                _ => Err(Error::Syntax),
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_signed()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_signed()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_signed()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()?)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse a string, check that it is one character, call `visit_char`.
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse the opening bracket of the sequence.
        if self.next_char()? == '{' {
            // Give the visitor access to each element of the sequence.
            let value = visitor.visit_seq(CommaSeparated::new(&mut self))?;
            // Parse the closing bracket of the sequence.
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedArray)
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next_char()? == '{' {
            let value = visitor.visit_map(ArmaClass::new(&mut self))?;
            Ok(value)
        } else {
            Err(Error::ExpectedMap)
        }
    }

    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek_char()? == '{' {
            self.next_char()?;
        }
        self.next_is_class = false;
        visitor.visit_map(ArmaClass::new(&mut self))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
        /*if self.peek_char()? == '"' {
            // Visit a unit variant.
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        } else if self.next_char()? == '{' {
            // Visit a newtype variant, tuple variant, or struct variant.
            let value = visitor.visit_enum(Enum::new(self))?;
            // Parse the matching close brace.
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedEnum)
        }*/
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

///////////////////

#[test]
fn test_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: u32,
        string: String,
    }

    let j = r#"int = 123;
string = "Hello";
"#;
    let expected = Test {
        int: 123,
        string: "Hello".to_string(),
    };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_escape() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        escape: String,
    }

    let j = r#"escape = "Hello ""World""";"#;
    let expected = Test {
        escape: "Hello \"World\"".to_string(),
    };
    assert_eq!(expected, from_str(j).unwrap());
}


#[test]
fn test_array() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        numbers: Vec<u8>,
        after: String,
    }

    let j = r#"numbers[] = {1,2,3};after="hi";"#;
    let expected = Test {
        numbers: vec![1,2,3],
        after: "hi".to_string(),
    };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_array_newline() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        numbers: Vec<u8>,
        after: String,
    }

    let j = r#"numbers[]=
{
    1,
    2,
    3
};
after="hi";"#;
    let expected = Test {
        numbers: vec![1,2,3],
        after: "hi".to_string(),
    };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_class_newline() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        numbers: Vec<u8>,
        after: String,
        child: Child,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct Child {
        number: u32,
        string: String,
    }

    let j = r#"numbers[] = {1,2,3};after="hi";
class child
{
    number= 123;
    string ="Hello";
};
    "#;
    let expected = Test {
        numbers: vec![1,2,3],
        after: "hi".to_string(),
        child: Child {
            number: 123,
            string: "Hello".to_string(),
        }
    };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_class_sameline() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        numbers: Vec<u8>,
        after: String,
        child: Child,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct Child {
        number: u32,
        string: String,
    }

    let j = r#"numbers[] = {1,2,3};after="hi";
class child {
    number= 123;
    string ="Hello";
};
    "#;
    let expected = Test {
        numbers: vec![1,2,3],
        after: "hi".to_string(),
        child: Child {
            number: 123,
            string: "Hello".to_string(),
        }
    };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_class_empty() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        numbers: Vec<u8>,
        after: String,
        child: Child,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct Child {}

    let j = r#"numbers[] = {1,2,3};after="hi";class child{};"#;
    let expected = Test {
        numbers: vec![1,2,3],
        after: "hi".to_string(),
        child: Child {}
    };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_dumb_newline() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        string: String,
    }

    let j = r#"string = "this is so dumb" \n "why would you do this";"#;
    let expected = Test {
        string: "this is so dumb\nwhy would you do this".to_string(),
    };
    assert_eq!(expected, from_str(j).unwrap());
}
