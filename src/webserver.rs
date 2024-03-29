use chrono::prelude::DateTime;
use chrono::Utc;
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

#[get("/?<keywords>")]
fn root(keywords: Option<String>) -> Html<String> {
    let conn = Connection::open(getenv("TWEED_DB_PATH")).unwrap();

    // Get the sentiments from the database.
    let mut stmt = conn
        .prepare(
            "
        select timestamp, keyword, score
        from avg_sentiments
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

    // Filter out keywords if the keywords parameter is specified.
    let keywords = keywords.unwrap_or("".to_string());
    let keywords: Vec<&str> = if keywords == "" {
        vec![]
    } else {
        keywords.split(",").collect()
    };

    // Gather up mapping of keyword -> [(x, y)]
    let mut keys_to_xs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut keys_to_ys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for s in sentiments {
        let s2 = s.unwrap();
        // Filter out tweets not in the list of keywords if some have been specified.
        let kw: &str = &s2.keyword.to_string();
        if keywords.len() > 0 && !keywords.contains(&kw) {
            continue;
        }
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
                x: [{}],
                y: [{}]
            }},
",
            k, xs_str, ys_str,
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

struct Sentiment {
    timestamp: i64,
    keyword: String,
    score: f64,
}
