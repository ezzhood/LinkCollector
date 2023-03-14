# LinkCollector
⚡ A lightning-fast web-crawler which collects links of a given host recursively and categorizes them to internal and external links. 
## Features
- **Parallel work** get queries faster with parallel works when websites have much more pages to request
- **RESTful API** run the server in background and integrate it in your technical stack

## Getting started
For getting started you will need [rust](https://www.rust-lang.org) and [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed on your computer. Then follow commands below:

To run

```bash
cargo run
```

To build:

```bash
cargo build --release
```

after building, you will have binary file located in `target/release/link_collector`, then just execute it.

Server address: `http://0.0.0.0:4000`

Available requests

-   `http://0.0.0.0:4000/links?url=https://my.website.com`

Query `url` takes seed host to look up links for.

## Demo

After running server, request to `http://0.0.0.0:4000/links?url=https://www.rust-lang.org` to get all links from official rust programming language. Result below:

<image src="assets/demo.png">

## Contributing

LinkCollector is open-source, so if you have any idea to improve or want to contribute feel free to open pull requests!
