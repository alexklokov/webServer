use std::collections::HashMap;

#[derive(Clone)]
pub struct Request {
    pub method: String,
    pub params: HashMap<String, String>,
}

impl Request {
    pub fn new(method: &str, params: HashMap<String, String>) -> Request {
        Request {
            method: String::from(method),
            params: params,
        }
    }
}
        // for (key, val) in self.paths.iter_mut() {
        //     if *key == url {
        //         *val = handler;
        //         return;
        //     }
        // }
        // self.paths.insert(url, handler);


#[derive(Clone)]
pub struct Router {
    pub paths: HashMap<&'static str, fn(&Request) -> Option<String>>,
}


impl Router {
    pub fn new() -> Router {
        Router {
            paths: HashMap::new(),
        }
    }

    pub fn add_path(&mut self, url: &'static str, handler: fn(&Request) -> Option<String>) {
        self.paths
            .entry(url)
            .and_modify(|val| *val = handler)
            .or_insert(handler);
    }

    pub fn remove_path(&mut self, url: &'static str) {
        self.paths.remove(url);
    }
}


