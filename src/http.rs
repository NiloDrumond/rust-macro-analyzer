use std::sync::Arc;

use tide::prelude::*;
use tide::security::CorsMiddleware;
use tide::security::Origin;
use tide::Body;
use tide::Request;

use crate::results::AnalyzisResults;
use crate::utils::pretty_print;

pub async fn start_server(results: AnalyzisResults) -> tide::Result<()> {
    let mut app = tide::new();

    let cors = CorsMiddleware::new()
        .allow_origin(Origin::from("*")) // You can specify your frontend's URL instead of "*" for better security
        // .allow_methods("GET".parse::<HeaderValue>().unwrap())
        .allow_credentials(false);

    app.with(cors);

    let results = Arc::new(results);

    app.at("/data").get(move |_: Request<()>| {
        let results = Arc::clone(&results);
        async move { Body::from_json(&results) }
    });
    pretty_print("HTTP Server started at port", Some(&"8080"));
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

// app.at("/repos").get(move |_: Request<()>| {
//     let results = Arc::clone(&results_clone);
//     async move { Body::from_json(&results.repos) }
// });
// let results_clone = results.clone();
// app.at("/crates").get(move |_: Request<()>| {
//     let results = Arc::clone(&results_clone);
//     async move { Body::from_json(&results.crates) }
// });
