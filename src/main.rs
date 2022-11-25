use actix_web::{web, App, HttpResponse, HttpServer};
use scraper::{Html, Selector};
use std::cmp::min;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::{self, available_parallelism};
use std::time::Instant;
#[derive(serde::Serialize)]
struct LinkResponse {
    internal_links: Vec<String>,
    external_links: Vec<String>,
    time_elapsed: u64,
}

#[derive(serde::Deserialize)]
struct LinkQuery {
    url: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server =
        HttpServer::new(move || App::new().route("/links", web::get().to(links_get_handler)))
            // .workers(1)
            .listen(TcpListener::bind("0.0.0.0:4000").unwrap())?
            .run();

    server.await
}

async fn links_get_handler(query: web::Query<LinkQuery>) -> HttpResponse {
    // maximum number of threads to know how many parallel works can be done
    let max_thread_number = available_parallelism().unwrap().get();
    // host is prefixed in parallel task so value of it shared among all threads
    let host = Arc::new(Mutex::new(query.url.to_owned()));

    // internal links are read among all threads
    let internal_links = Arc::new(Mutex::new(vec![host.lock().unwrap().to_owned()]));
    // external links are mutated in parallel job
    let external_links = Arc::new(Mutex::new(vec![]));

    let mut has_finished = false;
    let mut pointer = 0;

    let start_time = Instant::now();

    while !has_finished {
        let mut thread_handlers = vec![];
        let internal_links = Arc::clone(&internal_links);

        let new_found_internals: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let mut internal_links_vec = internal_links.lock().unwrap();

        let loop_number = min(max_thread_number, internal_links_vec.len() - pointer);

        for i in 0..loop_number {
            if let Some(url) = internal_links_vec.get(pointer + i) {
                let external_links = Arc::clone(&external_links);
                let new_found_internals = Arc::clone(&new_found_internals);
                let host = Arc::clone(&host);
                let url = url.clone();

                let handle = thread::spawn(move || {
                    let res_text = reqwest::blocking::get(url).unwrap().text().unwrap();

                    let (new_internals, new_externals) =
                        get_links_from_url(&host.lock().unwrap(), res_text);

                    let mut external_links_vec = external_links.lock().unwrap();

                    for new_external_link in new_externals {
                        if !external_links_vec.contains(&new_external_link) {
                            external_links_vec.push(new_external_link);
                        }
                    }

                    let mut new_found_internals_vec = new_found_internals.lock().unwrap();

                    for new_internal_link in new_internals {
                        new_found_internals_vec.push(new_internal_link);
                    }
                });

                thread_handlers.push(handle);
            } else {
                continue;
            }
        }

        for handle in thread_handlers {
            handle.join().unwrap();
        }

        pointer = internal_links_vec.len();

        let new_found_internals_vec = new_found_internals.lock().unwrap();

        for new_internal_link in new_found_internals_vec.iter() {
            if !internal_links_vec.contains(&new_internal_link) {
                internal_links_vec.push(new_internal_link.to_owned());
            }
        }

        if pointer == internal_links_vec.len() {
            has_finished = true;
        }
    }

    let result = LinkResponse {
        internal_links: internal_links.lock().unwrap().to_owned(),
        external_links: external_links.lock().unwrap().to_owned(),
        time_elapsed: start_time.elapsed().as_secs(),
    };

    HttpResponse::Ok().json(result)
}

fn get_links_from_url(host: &str, html_text: String) -> (Vec<String>, Vec<String>) {
    let mut external_links: Vec<String> = vec![];
    let mut internal_links: Vec<String> = vec![];

    Html::parse_document(&html_text)
        .select(&Selector::parse("a").unwrap())
        .filter_map(|n| n.value().attr("href"))
        .for_each(|link| {
            if link.starts_with("http") {
                external_links.push(String::from(link))
            } else {
                internal_links.push(host.to_owned() + link)
            }
        });

    (internal_links, external_links)
}
