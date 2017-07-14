# sesame-rs [![Build Status](https://travis-ci.org/MichaeGon/sesame-rs.svg?branch=master)](https://travis-ci.org/MichaeGon/sesame-rs) [![Crates.io](https://img.shields.io/crates/v/sesame_rs.svg)](https://crates.io/crates/sesame_rs)

Rust Client for [Sesame](https://candyhouse.co) made by CANDY HOUSE, Inc.

## Installation
Add this to your `Cargo.toml`
```toml
[dependencies]
sesame_rs = "*"
```

## Usage
```rust
extern crate sesame_rs;

use sesame_rs::*;

fn main() {

    // Initialize client
    let client = SesameClient::new();

    // Login to your account
    client.login(email, password).ok();

    for mut sesame in client.list_sesames().unwrap() {
        // Lock your sesame
        sesame.lock().ok();

        // Unlock your sesame
        sesame.unlock().ok();
    }
}

```
