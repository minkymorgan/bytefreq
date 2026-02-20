use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

lazy_static! {
    pub static ref HANDLE_COUNTRY_NAME_CACHE: RwLock<HashMap<String, Option<(String, String)>>> =
        RwLock::new(HashMap::new());
    pub static ref COUNTRY_NAME_TO_ISO3_CACHE: RwLock<HashMap<String, Option<String>>> =
        RwLock::new(HashMap::new());
}

