use chrono::DateTime;
use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use sentiment::analyze;
use serde::{Deserialize, Serialize};
use tokio::runtime::current_thread::block_on_all;
use twitter_stream::rt::{Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

/// streams tweets from the Twitter API, scores their sentiments
/// and stores the sentiments on their keywords in the sentiments table
/// in a SQLite database at tweed_db_path.
pub fn eat_tweets(tweed_db_path: String, keywords: Vec<String>, token: Token) {
    let conn = Connection::open(tweed_db_path).unwrap();
    conn.execute(
        "create table if not exists sentiments(
            timestamp integer not null,
            keyword text not null,
            score float not null
        )",
        NO_PARAMS,
    )
    .unwrap();

    let kwstr: &str = &keywords.join(",").to_string();

    let future = TwitterStreamBuilder::filter(token)
        .track(Some(kwstr))
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
                        let kwstr: &str = &kw.to_string();
                        let ts = parse_tweet_datetime(&t.created_at.to_string());
                        let tss: &str = &format!("{}", ts).to_string();
                        if t.text.contains(kwstr) {
                            conn.execute(
                                "insert into sentiments (timestamp, keyword, score)
                                     values (?1, ?2, ?3)
                                    ",
                                &[tss, &kw, &format!("{}", score)],
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

fn parse_tweet_datetime(dts: &str) -> i64 {
    let fmt = "%a %b %d %H:%M:%S %z %Y";
    let dt = DateTime::parse_from_str(dts, fmt).unwrap();
    dt.timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tweet_datetime() {
        assert_eq!(
            parse_tweet_datetime("Wed Oct 10 20:19:24 +0000 2018"),
            1539202764
        );
    }
}
