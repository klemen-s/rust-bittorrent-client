use serde_json;
use std::{env, panic};

enum EncodedBencodeValue {
    String,
    Number,
    List,
    Error,
}

fn decode_bencode_to_type(next_char: &char) -> EncodedBencodeValue {
    let is_string: bool = next_char.is_digit(10);
    let is_number: bool = *next_char == 'i';
    let is_list: bool = *next_char == 'l';

    if is_string {
        EncodedBencodeValue::String
    } else if is_number {
        EncodedBencodeValue::Number
    } else if is_list {
        EncodedBencodeValue::List
    } else {
        EncodedBencodeValue::Error
    }
}

fn decode_bencoded_string(encoded_value: &str) -> (serde_json::Value, &str) {
    if let Some((len, rest)) = encoded_value.split_once(":") {
        if let Ok(len) = len.parse::<usize>() {
            return (rest[..len].to_string().into(), &rest[len..]);
        } else {
            panic!("String length decoding failed")
        }
    } else {
        panic!("Bencode string decoding failed")
    }
}

fn decode_bencoded_number(encoded_value: &str) -> (serde_json::Value, &str) {
    let value = encoded_value.split_at(1).1;
    if let Some((number, rest)) = value.split_once("e") {
        if let Ok(number) = number.parse::<i64>() {
            (number.into(), rest)
        } else {
            panic!("Converting string to number failed!");
        }
    } else {
        panic!("Bencode number decoding failed!")
    }
}

fn decode_bencoded_list(encoded_value: &str) -> (serde_json::Value, &str) {
    let mut values = Vec::new();

    let mut rest = encoded_value.split_at(1).1;
    while !rest.is_empty() && !rest.starts_with('e') {
        let (v, remainder) = decode_bencoded_value(rest);
        values.push(v);
        rest = remainder;
    }

    (values.into(), rest)
}
#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    let mut chars = encoded_value.chars().peekable();

    let next_char = chars.peek().unwrap();
    match decode_bencode_to_type(next_char) {
        EncodedBencodeValue::String => decode_bencoded_string(encoded_value),
        EncodedBencodeValue::Number => decode_bencoded_number(encoded_value),
        EncodedBencodeValue::List => decode_bencoded_list(encoded_value),
        EncodedBencodeValue::Error => panic!("Unhandled encoded value: {}", encoded_value),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command: &String = &args[1];

    if command == "decode" {
        let encoded_value: &String = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.0.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
