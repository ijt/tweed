use sentiment::analyze;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::process::exit;
use tokio::runtime::current_thread::block_on_all;
use twitter_stream::rt::{Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

fn main() {
    let token = Token::new(
        getenv("CONSUMER_KEY"),
        getenv("CONSUMER_SECRET"),
        getenv("ACCESS_KEY"),
        getenv("ACCESS_SECRET"),
    );

    let args: Vec<String> = env::args().collect();
    if args.len() != 1 + 1 {
        println!(
            "usage: tweed keywords

where keywords is a comma-separated list of topic keywords
"
        );
        exit(1);
    }
    let keywords_str: &str = &args[1];
    let keywords: Vec<&str> = keywords_str.clone().split(",").collect();

    let mut kw_sentiment: HashMap<String, f32> = HashMap::new();
    let mut kw_count: HashMap<String, i32> = HashMap::new();

    let future = TwitterStreamBuilder::filter(token)
        .track(Some(keywords_str))
        .listen()
        .unwrap()
        .flatten_stream()
        .take(5)
        .for_each(|json| {
            let tr: serde_json::Result<Tweet> = serde_json::from_str(&json.to_string());
            match tr {
                Err(_e) => (),
                Ok(t) => {
                    let score = analyze(t.text.clone()).comparative;
                    println!("{}: {}\n", score, t.text);
                    for kw in keywords.clone() {
                        let kw2: String = kw.to_string();
                        if t.text.contains(kw) {
                            *kw_sentiment.entry(kw2.clone()).or_insert(0.0f32) += score;
                            *kw_count.entry(kw2).or_insert(0i32) += 1
                        }
                    }
                }
            }

            Ok(())
        })
        .map_err(|e| println!("error: {}", e));

    if let Err(e) = block_on_all(future) {
        println!("Stream error: {:?}", e);
        println!("Disconnected")
    }

    for kw in keywords {
        let s = kw_sentiment.get(kw).unwrap_or(&0.0f32);
        let n = kw_count.get(kw).unwrap_or(&0i32);
        let s2 = s / (*n as f32);
        println!("{}: {}", kw, s2);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Tweet {
    created_at: String,
    text: String,
}

fn getenv(s: &str) -> String {
    match env::var(s) {
        Ok(v) => v,
        Err(_) => {
            println!("${} not defined", s);
            exit(1);
        }
    }
}
