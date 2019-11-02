#![feature(proc_macro_hygiene, decl_macro)]

mod tweeteater;
mod webserver;
use std::thread;

fn main() {
    let h1 = thread::spawn(tweeteater::eat_tweets);
    let h2 = thread::spawn(webserver::serve_plots);
    h1.join().unwrap();
    h2.join().unwrap();
}
