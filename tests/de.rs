use serde::Deserialize;

use serde_arma;
mod mission;

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
    assert_eq!(expected, serde_arma::from_str(j).unwrap());
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
    assert_eq!(expected, serde_arma::from_str(j).unwrap());
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
    assert_eq!(expected, serde_arma::from_str(j).unwrap());
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
    assert_eq!(expected, serde_arma::from_str(j).unwrap());
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
    assert_eq!(expected, serde_arma::from_str(j).unwrap());
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
    assert_eq!(expected, serde_arma::from_str(j).unwrap());
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
    assert_eq!(expected, serde_arma::from_str(j).unwrap());
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
    assert_eq!(expected, serde_arma::from_str(j).unwrap());
}

#[test]
fn test_transcode() {
    use serde_json::{Serializer};
    use std::io::{Write, BufReader, BufWriter};
    use std::fs::File;

    let reader = BufReader::new(File::open("tests/example.hpp").unwrap());
    let writer = BufWriter::new(File::create("tests/example.json").unwrap());

    let mut deserializer = serde_arma::from_reader(reader);
    let mut serializer = Serializer::pretty(writer);
    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();
    serializer.into_inner().flush().unwrap();
}

#[test]
fn test_mission() {
    use std::fs;
    let contents = fs::read_to_string("tests/example.hpp")
        .expect("Something went wrong reading the file");

    let _: crate::mission::InternalArmaMission = serde_arma::from_str(&contents).unwrap();
}
