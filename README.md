# tsurch
A Rust CLI tool for searching the web

## Features

Running with a single command-line parameter (one word or a sequence of words surrounded by double quotes) will send semi-terminal-friendly search results
from DuckDuckGo to standard output. You can use `-s=...` or `--source=...` to select a different search source; currently included are

- [`duckduckgo`](https://duckduckgo.com/), aliased as `duck` and `ddg`
- [`startpage`](https://startpage.com/), aliased as `start` and `sp` (**does not work**)
- [`wikipedia`](https://en.wikipedia.org/), aliased as `wiki` and `wp`
- [`rustdoc`](https://doc.rust-lang.org/std/), aliased as `rdoc` and `rs` (**does not work**)

## Running

Run with `cargo run` while in the project directory.
An executable will be created in `target/debug` (or `target/release` if you run with `--release`), which you can run independently of `cargo`,
and perhaps alias for quicker access.
