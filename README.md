# tweed: a sentiment plotter for Twitter tweets

## System structure

The browser side is done with [plot.ly](https://plot.ly/javascript/).

The backend is a Rust program that runs a webserver to generate plots plus a thread called tweeteater that pulls
tweets from Twitter, computes their sentiment scores and saves the scores to a SQLite database.

Here is a diagram showing how the information flows:

```
|Twitter API| => |tweeteater thread| => |sentiments table in SQLite db|
    => |averager thread| => |avg_sentiments table in SQLite db|
    => |webserver thread| => |plotly in browser|
```

### Schema

The schema for the time series is as follows:

```sql
create table sentiments(
	timestamp integer,
	keyword text,
	score float,
)
```

Each row contains the sentiment for a single tweet containing a keyword.
A single tweet may contain more than one keyword, so it may result in
multiple rows in this table.

### URL parameters

* keywords: comma-separated list of keywords to plot, default "", meaning all of them
* sigma: amount of jitter to add to prevent points from overlapping, e.g., 0.001, default 0

## How to run it

Start by getting your Twitter development credentials, then plug them into this incantation:
```
$ CONSUMER_KEY=<consumer key> \
CONSUMER_SECRET=<consumer secret> \
ACCESS_KEY=<access key> \
ACCESS_SECRET=<access secret> \
TWEED_DB_PATH=tweed.db \
cargo run dogs,cats,monkeys
```
where `dogs,cats,monkeys` is one example of tweet keywords to track.

You can see the result by visiting `http://localhost:8000` and refreshing to get the latest data as it comes in.
