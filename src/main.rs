use actix_web::{web, App, HttpResponse, HttpServer};
use select::{document::Document, predicate::Name};
use std::net::TcpListener;

#[derive(serde::Serialize)]
struct LinkResponse {
    internal_links: Vec<String>,
    external_links: Vec<String>,
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
    let client = reqwest::Client::new();

    let mut has_finished = false;
    let mut pointer = 0;

    let mut internal_links: Vec<String> = vec![host.to_owned()];
    let mut external_links: Vec<String> = vec![];

    while !has_finished {
        let res_text = client
            .get(internal_links.get(pointer).unwrap())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let (new_internals, new_externals) = get_links_from_url(res_text);

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

    // let links = get_links_from_url(res_text);

    let result = LinkResponse {
        internal_links,
        external_links,
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
