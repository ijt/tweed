use chrono::prelude::DateTime;
use chrono::Utc;
use rand::distributions::{Distribution, Normal};
use rocket::response::content::Html;
use rocket::{get, routes};
use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::process::exit;
use std::time::{Duration, UNIX_EPOCH};

pub fn serve_plots() {
    rocket::ignite().mount("/", routes![root]).launch();
}

/// The sigma parameter specifies the standard deviation of some jitter for the scatter points
/// so they don't overlap as much.
#[get("/?<sigma>&<size>")]
fn root(sigma: Option<f64>, size: Option<i32>) -> Html<String> {
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
    let mut keys_to_xs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut keys_to_ys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for s in sentiments {
        let s2 = s.unwrap();
        let d = UNIX_EPOCH + Duration::from_secs(s2.timestamp as u64);
        let datetime = DateTime::<Utc>::from(d);
        let timestamp_str = datetime.format("'%Y-%m-%d %H:%M:%S'").to_string();
        let x = format!("{}", timestamp_str);
        let noise = Normal::new(0.0f64, sigma.unwrap_or(0.0f64)).sample(&mut rand::thread_rng());
        let y = format!("{}", s2.score + noise);
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
                marker: {{ size: {size} }},
                type: \"scatter\",
                x: [{}],
                y: [{}]
            }},
",
            k,
            xs_str,
            ys_str,
            size = size.unwrap_or(4i32)
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
