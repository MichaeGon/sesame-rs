pub const PREFIX: &'static str = "https://api.candyhouse.co/v1";
pub const LOGIN_ENDPOINT: &'static str  = "/accounts/login";
pub const SESAME_ENDPOINT: &'static str = "/sesames";
pub const CONTROL_ENDPOINT: &'static str = "/control";

/// X-Authorization header
header! { (XAuth, "X-Authorization") => [String] }
