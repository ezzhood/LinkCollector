To run

```bash
    cargo run
```

To build:

```bash
    cargo build --release
```

after building

```bash
    cd target/release
    ./link_rust
```

Server address: `http://0.0.0.0:4000`

Available requests

-   `http://0.0.0.0:4000/links?url=https://my.website.com`

Query takes `url` to look up links for.
