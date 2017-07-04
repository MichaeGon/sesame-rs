# sesame-rs [![Build Status](https://travis-ci.org/MichaeGon/sesame-rs.svg?branch=master)](https://travis-ci.org/MichaeGon/sesame-rs) [![Crates.io](https://img.shields.io/crates/v/sesame_rs.svg)](https://crates.io/crates/sesame_rs)

Rust Client for [Sesame](https://candyhouse.co) made by CANDY HOUSE, Inc.


## Usage
```rust
extern crate sesame_rs;
extern crate rpassword;

use sesame_rs::*;
use rpassword::read_password;

fn main() {
    println!("email:");
    let email = get_line();
    println!("password:");
    let pass = read_password().unwrap();

    // initialize client
    let sesame = SesameClient::new();
    // login
    sesame.login(email, pass).ok();

    for mut s in sesame.list_sesames().unwrap() {
        // lock sesame
        s.lock().ok();
    }
}

fn get_line() -> String {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).ok();
    s.trim().to_string()
}

```
