# Rust Roguelike Tutorial

This is a [Roguelike Tutorial - in Rust](https://github.com/thebracket/rustrogueliketutorial) implemented using [Legion](https://github.com/TomGillen/legion) ECS (instead of Specs) and [RON](https://github.com/ron-rs/ron) based prefabs (instead of JSON).

Still work in progress - as I progress through the tutorial.

## Running

Unfortunately, it requires `master` branch of Legion, so you will need to checkout `legion` submodule.

Either clone all together:

    git clone --recursive https://github.com/smokku/rust_roguelike_tutorial.git

or after normal clone do:

    git submodule update --init --recursive

## Building for Web

    cargo +nightly -Z features=itarget build --release --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/release/rust_roguelike_tutorial.wasm --out-dir web --no-modules --no-typescript

    serve web/
