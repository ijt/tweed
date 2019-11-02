# tweed: a tweet sentiment plotter

## System structure

The browser side is done with plot.ly. It keeps a websocket open to the server
side to stream the latest sentiment values.

The backend is a Rust program that runs a webserver to generate plots plus a thread that pulls
tweets from Twitter, computes their sentiment scores and saves the scores to a SQLite database.

### Schema

The schema for the time series is something like this:

```sql
create table sentiments(
	timestamp integer,
	keyword text,
	sentiment float,
)
```

Each row contains the sentiment for a single tweet containing a keyword.
A single tweet may contain more than one keyword, so it may result in
multiple rows in this table.

### URL scheme

`http://tweed.best` shows sentiments over the past day
`http://tweed.best?days=2` shows the sentiments for tweets over the past 2 days.

### Plotting

`http://tweed.best?avg=1h` plots using an hour-long averaging window. The averaging is done on the server side to
minimize the amount of data sent over the wire.

## Deployment

```
cargo build --target=x86_64-unknown-linux-gnu --features 'standalone'
```
Copy `target/x86_64-unknown-linux-gnu/release/tweelings` to a VM in the cloud and run it there under upstart.

FIXME: add details.

