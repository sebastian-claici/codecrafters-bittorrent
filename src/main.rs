use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    // If encoded_value starts with a digit, it's a number
    match encoded_value.chars().next() {
        Some('0'..='9') => {
            if let Some((len, rest)) = encoded_value.split_once(':') {
                if let Ok(n) = len.parse::<usize>() {
                    return (rest[..n].into(), &rest[n..]);
                }
            }
        }
        Some('i') => {
            if let Some((n, rest)) = encoded_value
                .split_once('i')
                .and_then(|(_, rest)| rest.split_once('e'))
                .and_then(|(n, rest)| {
                    let n = n.parse::<i64>().expect("Expected integer");
                    Some((n, rest))
                })
            {
                return (n.into(), rest);
            }
        }
        Some('l') => {
            let mut values: Vec<serde_json::Value> = Vec::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (value, remainder) = decode_bencoded_value(&rest);
                values.push(value);
                rest = &remainder;
            }
            return (values.into(), rest);
        }
        Some('d') => {
            let mut dict = serde_json::Map::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (key, remainder) = decode_bencoded_value(&rest);
                let (value, remainder) = decode_bencoded_value(&remainder);
                let key = match key {
                    serde_json::Value::String(key) => key,
                    _ => panic!("Dict keys must be strings")
                };
                dict.insert(key, value);
                rest = &remainder;
            }
            return (dict.into(), rest);
        }
        _ => {}
    }

    panic!("Unhandled encoded value: {}", encoded_value);
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.0.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
