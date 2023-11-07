# Globe

## About

An implementation of a rotating globe in **Rust** **WebAssembly**.

![Image of globe](./images/output.png)

Data sourced from [Natural Earth](https://www.naturalearthdata.com/) is transformed into **Rust** code during compilation using a [build script](./build.rs).

## Prerequisites

Install [**Rust**](https://www.rust-lang.org/) and [**wasm-pack**](https://github.com/rustwasm/wasm-pack).

## Build

```bash
wasm-pack build --target web
```
or optimised for release
```bash
wasm-pack build --target web --release
```

## Run

Some options to serve the application include:
```bash
# Python 3.x
python3 -m http.server
# Python 2.x
python -m SimpleHTTPServer
# JDK 18 or later
jwebserver
```

Access via a web browser at [http://localhost:8000](http://localhost:8000).
