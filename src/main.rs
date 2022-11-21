use actix_web::{web, App, HttpResponse, HttpServer};
use select::{document::Document, predicate::Name};
use std::net::TcpListener;
use std::sync::mpsc::channel;
use std::thread::{self, available_parallelism};
use std::time::Instant;

#[derive(serde::Serialize)]
struct LinkResponse {
    internal_links: Vec<String>,
    external_links: Vec<String>,
    time_elapsed: u64,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server =
        HttpServer::new(move || App::new().route("/links", web::get().to(links_get_handler)))
            .listen(TcpListener::bind("0.0.0.0:4000").unwrap())?
            .run();

    server.await
}

async fn links_get_handler() -> HttpResponse {
    let host = "https://serpay.penjire.com";
    let thread_max_number = available_parallelism().unwrap().get();

    let mut has_finished = false;
    let mut pointer = 0;

    let mut internal_links: Vec<String> = vec![host.to_owned()];
    let mut external_links: Vec<String> = vec![];

    let (tx, rx) = channel();

    let start_time = Instant::now();

    while !has_finished {
        let mut recieve_count = 0;

        for i in 0..thread_max_number {
            let tx = tx.clone();

            if let Some(url) = internal_links.get(pointer + i) {
                let url = url.clone();

                thread::spawn(move || {
                    let res_text = reqwest::blocking::get(url).unwrap().text().unwrap();

                    let internals_tuple = get_links_from_url(res_text);
                    tx.send(internals_tuple).unwrap();
                });
                recieve_count += 1;
            } else {
                break;
            }
        }

        for _ in 0..recieve_count {
            if let Ok((new_internals, new_externals)) = rx.recv() {
                for new_external_link in new_externals {
                    if !external_links.contains(&new_external_link) {
                        external_links.push(new_external_link);
                    }
                }

                for new_internal_link in new_internals {
                    if !internal_links.contains(&new_internal_link) {
                        internal_links.push(new_internal_link);
                    }
                }

                if pointer == internal_links.len() - 1 {
                    has_finished = true;
                } else {
                    pointer += 1;
                }
            }
        }
    }

    let result = LinkResponse {
        internal_links,
        external_links,
        time_elapsed: start_time.elapsed().as_secs(),
    };

    // println!("{:#?}", res_text);

    HttpResponse::Ok().json(result)
}

fn get_links_from_url(html_text: String) -> (Vec<String>, Vec<String>) {
    let mut external_links: Vec<String> = vec![];
    let mut internal_links: Vec<String> = vec![];
    Document::from(html_text.as_str())
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(|x| x)
        .for_each(|link| {
            if link.starts_with("http") {
                external_links.push(String::from(link))
            } else if link != "/" {
                internal_links.push("https://serpay.penjire.com".to_owned() + link)
            }
        });

    (internal_links, external_links)
}
