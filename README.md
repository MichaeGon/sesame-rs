# sesame-rs

Rust Client for [Sesame](https://candyhouse.co) made by CANDY HOUSE, Inc.

## Usage
```rust
extern crate sesame_rs;
extern crate rpassword;

use sesame_rs::*;

fn main() {
    println!("email:");
    let email = get_line();
    println!("password:");
    let pass = read_password().unwrap();

    // initialize client
    let sesame = sesame_rs::SesameClient::new();
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
