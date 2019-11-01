use twitter_stream::{Token, TwitterStreamBuilder};
use twitter_stream::rt::{self, Future, Stream};

fn main() {
    let token = Token::new("consumer_key", "consumer_secret", "access_key", "access_secret");

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
