# tweelings: a Twitter sentiment grapher

## System structure

The browser side is done with plot.ly. It keeps a websocket open to the server
side to stream the latest sentiment values.

The backend is a Rust program that runs a webserver plus a thread that pulls
tweets from Twitter and saves their time-averaged sentiment scores to a SQLite
database.

### Schema

The schema for the time series is something like this:

```sql
CREATE TABLE sentiments(
	timestamp INTEGER,
	keyword TEXT,
	sentiment FLOAT,
)
```

The sentiments are calculated as the average sentiment for the tweets for the
keyword over the sampling interval.

### Time-averaged sentiment

Two hashmaps are updated from the incoming tweets, one counting how many
tweets match each keyword and another adding up the sentiment values for
each keyword.

Periodically, the average sentiment for each keyword is computed from these
hashmaps and stored in the database. The hashmaps are then cleared for the
next interval.

## Deployment

```
cargo build --target=x86_64-unknown-linux-gnu --features 'standalone'
```
Copy `target/x86_64-unknown-linux-gnu/release/tweelings` to a VM in the cloud and run it there under upstart.

FIXME: add details.

