#![feature(proc_macro_hygiene, decl_macro)]

use chrono::prelude::DateTime;
use chrono::Utc;
use rocket::response::content::Html;
use rocket::{get, routes};
use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use sentiment::analyze;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::process::exit;
use std::thread;
use std::time::{Duration, UNIX_EPOCH};
use tokio::runtime::current_thread::block_on_all;
use twitter_stream::rt::{Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

fn main() {
    let h1 = thread::spawn(eat_tweets);
    let h2 = thread::spawn(serve_plots);
    h1.join().unwrap();
    h2.join().unwrap();
}

fn eat_tweets() {
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

fn serve_plots() {
    rocket::ignite().mount("/", routes![root]).launch();
}

#[get("/")]
fn root() -> Html<String> {
    let conn = Connection::open(getenv("TWEED_DB_PATH")).unwrap();

    // Get the sentiments from the database.
    let mut stmt = conn
        .prepare(
            "
        select timestamp, keyword, score
        from sentiments
    ",
        )
        .unwrap();
    let sentiments = stmt
        .query_map(NO_PARAMS, |row| {
            Ok(Sentiment {
                timestamp: row.get(0).unwrap(),
                keyword: row.get(1).unwrap(),
                score: row.get(2).unwrap(),
            })
        })
        .unwrap();

    // Gather up mapping of keyword -> [(x, y)]
    let mut keys_to_xs: HashMap<String, Vec<String>> = HashMap::new();
    let mut keys_to_ys: HashMap<String, Vec<String>> = HashMap::new();
    for s in sentiments {
        let s2 = s.unwrap();
        let d = UNIX_EPOCH + Duration::from_secs(s2.timestamp as u64);
        let datetime = DateTime::<Utc>::from(d);
        let timestamp_str = datetime.format("'%Y-%m-%d %H:%M:%S'").to_string();
        let x = format!("{}", timestamp_str);
        let y = format!("{}", s2.score);
        (*keys_to_xs.entry(s2.keyword.clone()).or_insert(vec![])).push(x);
        (*keys_to_ys.entry(s2.keyword).or_insert(vec![])).push(y);
    }

    // Output the HTML with the plot.
    let mut out = String::new();
    out.push_str(
        "
<html>
    <head>
        <script src=\"https://cdn.plot.ly/plotly-1.5.0.min.js\"></script>
    </head>
    <body>
    <div id=\"plot\" style=\"width:100%;height:100%;\"></div>
    <script>
        var p = document.getElementById('plot');

        var layout = {
          title: 'Tweet Sentiments',
          xaxis: { title: 'Time' },
          yaxis: { title: 'Sentiment' },
        };

        Plotly.plot( p, [",
    );

    for (k, xs) in &keys_to_xs {
        let ys = keys_to_ys.get(k).unwrap();
        let xs_str = xs.join(", ");
        let ys_str = ys.join(", ");
        let part = format!(
            "
            {{
                name: \"{}\",
                mode: \"markers\",
                type: \"scatter\",
                x: [{}],
                y: [{}]
            }},
",
            k, xs_str, ys_str
        );
        out.push_str(&part.to_string());
    }
    out.push_str(
        "
        ], layout);
    </script>
    ",
    );
    out.push_str(
        "
    </body>
</html>
",
    );

    Html(out)
}

struct Sentiment {
    timestamp: i64,
    keyword: String,
    score: f64,
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
