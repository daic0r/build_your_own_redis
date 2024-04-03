use std::collections::HashMap;

use super::ResponseStatus;

pub struct Database {
    data: HashMap<String, String>,
}

impl Database {
    pub fn new() -> Database {
        Database {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> ResponseStatus {
        self.data.insert(key, value);
        ResponseStatus::Ok
    }

    pub fn get(&self, key: &str, value: &mut String) -> ResponseStatus {
        match self.data.get(key) {
            Some(v) => {
                value.push_str(v);
                ResponseStatus::Ok
            }
            None => ResponseStatus::Nx,
        }
    }

    pub fn del(&mut self, key: &str) -> ResponseStatus {
        self.data.remove(key);
        ResponseStatus::Ok
    }
}
