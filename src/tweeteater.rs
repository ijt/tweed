use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use sentiment::analyze;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::exit;
use tokio::runtime::current_thread::block_on_all;
use twitter_stream::rt::{Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

pub fn eat_tweets() {
    let token = Token::new(
        getenv("CONSUMER_KEY"),
        getenv("CONSUMER_SECRET"),
        getenv("ACCESS_KEY"),
        getenv("ACCESS_SECRET"),
    );

    let conn = Connection::open(getenv("TWEED_DB_PATH")).unwrap();
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

    conn.execute(
        "create table if not exists sentiments(
            timestamp integer not null,
            keyword text not null,
            score float not null,
            tweet text not null
        )",
        NO_PARAMS,
    )
    .unwrap();

    let future = TwitterStreamBuilder::filter(token)
        .track(Some(keywords_str))
        .listen()
        .unwrap()
        .flatten_stream()
        .for_each(|json| {
            let tr: serde_json::Result<Tweet> = serde_json::from_str(&json.to_string());
            match tr {
                Err(_e) => (),
                Ok(t) => {
                    let score = analyze(t.text.clone()).comparative;
                    for kw in keywords.clone() {
                        let kw2: String = kw.to_string();
                        if t.text.contains(kw) {
                            conn.execute(
                                "insert into sentiments (timestamp, keyword, score, tweet)
                                     values (cast(strftime('%s', 'now') as int), ?1, ?2, ?3)
                                    ",
                                &[&kw2, &format!("{}", score), &t.text],
                            )
                            .unwrap();
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
