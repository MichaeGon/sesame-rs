/// JSON structures

#[derive(Serialize, Deserialize)]
pub struct SesameBody {
    pub nickname: String,
    pub is_unlocked: bool,
    pub api_enabled: bool,
    pub battery: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SesameListElem {
    pub device_id: String,
    pub nickname: String,
    pub is_unlocked: bool,
    pub api_enabled: bool,
    pub battery: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthToken {
    pub authorization: String,
}

#[derive(Serialize, Deserialize)]
pub struct SesameList {
    pub sesames: Vec<SesameListElem>,
}
