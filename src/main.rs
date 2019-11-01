use serde::{Deserialize, Serialize};
use std::env;
use std::process::exit;
use twitter_stream::rt::{self, Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

fn main() {
    let token = Token::new(
        env::var("CONSUMER_KEY").unwrap(),
        env::var("CONSUMER_SECRET").unwrap(),
        env::var("ACCESS_KEY").unwrap(),
        env::var("ACCESS_SECRET").unwrap(),
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
    let keywords: &str = &args[1].to_string();

    let future = TwitterStreamBuilder::filter(token)
        .track(Some(keywords))
        .listen()
        .unwrap()
        .flatten_stream()
        .for_each(|json| {
            let tr: serde_json::Result<Tweet> = serde_json::from_str(&json.to_string());
            match tr {
                Err(_e) => (),
                Ok(t) => println!("{}\n", t.text),
            }
            Ok(())
        })
        .map_err(|e| println!("error: {}", e));

    rt::run(future);
}

#[derive(Serialize, Deserialize, Debug)]
struct Tweet {
    created_at: String,
    text: String,
}
