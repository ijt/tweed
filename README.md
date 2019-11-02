# tweed: a sentiment plotter for Twitter tweets

## System structure

The browser side is done with [plot.ly](https://plot.ly/javascript/).

The backend is a Rust program that runs a webserver to generate plots plus a thread called tweeteater that pulls
tweets from Twitter, computes their sentiment scores and saves the scores to a SQLite database.

Here is a diagram showing how the information flows:

```
|Twitter API| => |tweeteater thread| => |SQLite db| => |webserver thread| => |plotly in browser|
```

### Schema

The schema for the time series is as follows:

```sql
create table sentiments(
	timestamp integer,
	keyword text,
	score float,
	tweet text,
)
```

Each row contains the sentiment for a single tweet containing a keyword.
A single tweet may contain more than one keyword, so it may result in
multiple rows in this table.

### URL parameters

sigma: amount of jitter to add to prevent points from overlapping, e.g., 0.001, default 0
size: point size in pixels, default 4
keywords: comma-separated list of keywords to plot, default "", meaning all of them

## Deployment

```
cargo build --target=x86_64-unknown-linux-gnu --features 'standalone'
```
Copy `target/x86_64-unknown-linux-gnu/release/tweed` to a VM in the cloud and run it there under upstart.

