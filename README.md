# tweelings: a Twitter sentiment grapher

## System structure

The browser side is done with plot.ly. It keeps a websocket open to the server
side to stream the latest sentiment values.

The backend is a Rust program that runs a webserver plus a thread that pulls
tweets from Twitter once per minute and saves them to a SQLite database.

## Deployment

```
cargo build --target=x86_64-unknown-linux-gnu --features 'standalone'
```
Copy `target/x86_64-unknown-linux-gnu/release/tweelings` to a VM in the cloud and run it there under upstart.

FIXME: add details.

