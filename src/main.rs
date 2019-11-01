use std::env;
use twitter_stream::rt::{self, Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

fn main() {
    let token = Token::new(
        env::var("CONSUMER_KEY").unwrap(),
        env::var("CONSUMER_SECRET").unwrap(),
        env::var("ACCESS_KEY").unwrap(),
        env::var("ACCESS_SECRET").unwrap(),
    );

    let future = TwitterStreamBuilder::filter(token)
        .track(Some("@Twitter"))
        .listen()
        .unwrap()
        .flatten_stream()
        .for_each(|json| {
            println!("{}", json);
            Ok(())
        })
        .map_err(|e| println!("error: {}", e));

    rt::run(future);
}
