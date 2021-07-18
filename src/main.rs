// The MIT License (MIT)
// Copyright © 2021 Aukbit Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

mod config;
mod errors;
mod crunch;

use crate::config::CONFIG;
use crate::crunch::Crunch;
use log::info;
use std::env;

fn main() {
    // Load configuration
    let config = CONFIG.clone();

    info!(
        "Starting {} version {} <> {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        config.substrate_ws_url
    );

    Crunch::it();
}

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     // Load configuration
//     let config = CONFIG.clone();

//     info!(
//         "Starting {} version {} <{}>",
//         env!("CARGO_PKG_NAME"),
//         env!("CARGO_PKG_VERSION"),
//         config.substrate_ws_url
//     );

//     // Spawn history and subscription sincronization tasks
//     Sync::run();

//     // Start http server
//     let addr = format!("{}:{}", config.turboflakes_host, config.turboflakes_port);
//     HttpServer::new(move || {
//         let cors = Cors::default()
//             .allowed_origin_fn(|origin, _req_head| {
//                 let allowed_origin =
//                     env::var("TURBOFLAKES_CORS_ALLOW_ORIGIN").unwrap_or("*".to_string());
//                 origin.as_bytes().ends_with(allowed_origin.as_bytes())
//             })
//             .allowed_methods(vec!["GET", "OPTIONS"])
//             .allowed_headers(vec![http::header::CONTENT_TYPE])
//             .supports_credentials()
//             .max_age(3600);
//         App::new()
//             .wrap(middleware::Logger::default())
//             .wrap(cors)
//             .configure(add_pool)
//             .configure(routes)
//     })
//     .bind(addr)?
//     .run()
//     .await
// }
