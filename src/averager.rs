use rusqlite::{params, Connection, NO_PARAMS};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

/// computes minute-wise averages for scores in the sentiments table, stores those in the
/// avg_sentiments table, and deletes the rows from the sentiments table that went into the
/// calculation.
pub fn average_sentiments(tweed_db_path: String, keywords: Vec<String>) {
    let mut conn = Connection::open(tweed_db_path).unwrap();
    conn.execute(
        "create table if not exists avg_sentiments(
            timestamp integer not null,
            keyword text not null,
            score float not null
        )",
        NO_PARAMS,
    )
    .unwrap();

    loop {
        // Figure out the most recently completed minute, call this tf.
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let tf = ((now.as_secs() / 60) * 60) as i64;

        for kw in &keywords {
            let mut minutes_totals: BTreeMap<Duration, f64> = BTreeMap::new();
            let mut minutes_counts: BTreeMap<Duration, u64> = BTreeMap::new();
            let mut get_totals_counts = || {
                let mut stmt = conn
                    .prepare(
                        "
                select timestamp, score
                from sentiments
                where timestamp < ?1
                and keyword = ?2
                order by timestamp",
                    )
                    .unwrap();
                let sentiments = stmt
                    .query_map(params![tf, kw], |row| {
                        Ok(Sentiment2 {
                            timestamp: row.get(0).unwrap(),
                            score: row.get(1).unwrap(),
                        })
                    })
                    .unwrap();

                // Make a map from minutes to scores seen within that minute.
                for s in sentiments {
                    let s = s.unwrap();
                    let t = (s.timestamp / 60) * 60; // truncate to minute
                    let d = Duration::new(t as u64, 0);
                    let total = minutes_totals.entry(d).or_insert(0.0f64);
                    let count = minutes_counts.entry(d).or_insert(0u64);
                    *total += s.score;
                    *count += 1;
                }
            };
            get_totals_counts();

            // Update the sentiments and avg_sentiments tables in a transaction.
            let update = || {
                let tx = conn.transaction().unwrap();

                // For each minute, compute its average, store it in the avg_sentiments table.
                for (d, total) in minutes_totals {
                    let count = minutes_counts.get(&d).unwrap();
                    let avg = total / (*count as f64);
                    tx.execute(
                        "insert into avg_sentiments (timestamp, keyword, score) values (?1, ?2, ?3)",
                        params![&(d.as_secs() as i64), &kw, &avg]).unwrap();
                }

                // Remove all the tweet-wise scores gathered earlier from the sentiments table.
                tx.execute("delete from sentiments where timestamp < ?1", params![tf])
                    .unwrap();

                tx.commit().unwrap();
            };
            update();
        }
    }
}

// This is called Sentiment2 because calling it Sentiment made it show up red in Intellij.
struct Sentiment2 {
    timestamp: i64,
    score: f64,
}