# Rust Roguelike Tutorial

This is a [Roguelike Tutorial - in Rust](https://github.com/thebracket/rustrogueliketutorial) implemented using Legion ECS (instead of SPECS).

Still work in progress - as I progress through the tutorial.

## Running

Unfortunately, it requires `master` branch of Legion, so you will need to checkout [legion](https://github.com/TomGillen/legion) as submodule.

    git submodule init
    git submodule update

## Building for Web

    cargo +nightly -Z features=itarget build --release --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/release/rust_roguelike_tutorial.wasm --out-dir web --no-modules --no-typescript

    serve web/
