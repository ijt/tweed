#![feature(proc_macro_hygiene, decl_macro)]

mod averager;
mod tweeteater;
mod webserver;
use rusqlite::{Connection, NO_PARAMS};
use std::env;
use std::process::exit;
use std::thread;
use twitter_stream::Token;

fn main() {
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
    // Make ownable versions of keywords to pass off to the threads, to work around
    // rust's worry that the threads might outlive main.
    let kws1: Vec<String> = keywords.iter().map(|s| s.to_string()).collect();
    let kws2: Vec<String> = keywords.iter().map(|s| s.to_string()).collect();

    // Also make two instances of tweed_db_path so threads can own them.
    let tweed_db_path1 = getenv("TWEED_DB_PATH");
    let tweed_db_path2 = getenv("TWEED_DB_PATH");

    let token = Token::new(
        getenv("CONSUMER_KEY"),
        getenv("CONSUMER_SECRET"),
        getenv("ACCESS_KEY"),
        getenv("ACCESS_SECRET"),
    );

    // Create tables if needed.
    let conn = Connection::open(getenv("TWEED_DB_PATH")).unwrap();
    conn.execute(
        "create table if not exists avg_sentiments(
            timestamp integer not null,
            keyword text not null,
            score float not null
        )",
        NO_PARAMS,
    )
    .unwrap();
    conn.execute(
        "create table if not exists sentiments(
            timestamp integer not null,
            keyword text not null,
            score float not null
        )",
        NO_PARAMS,
    )
    .unwrap();

    let h1 = thread::spawn(|| tweeteater::eat_tweets(tweed_db_path1, kws1, token));
    let h2 = thread::spawn(|| averager::average_sentiments(tweed_db_path2, kws2));
    let h3 = thread::spawn(webserver::serve_plots);
    h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();
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
