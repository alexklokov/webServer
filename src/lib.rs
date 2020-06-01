pub mod router;
use router::Request;
use router::Router;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

pub struct Server {
    pub ip: String,
    pub port: String,
}

impl Server {
    pub fn new(ip_address: &str, port: &str) -> Self {
        Server {
            ip: ip_address.to_string(),
            port: port.to_string(),
        }
    }

    pub fn start(&mut self, router: Router) {
        let current_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
        let listener = match TcpListener::bind(format!("{}:{}", self.ip, self.port)) {
            Ok(listener) => listener,
            Err(_) => panic!("Адрес недоступен"),
        };

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let clone_router = router.clone();
            let clone_current_dir = current_dir.clone();
            thread::spawn(move || {
                handler(stream, clone_router, clone_current_dir);
            });
        }
    }
}

fn handler(mut stream: TcpStream, router: Router, mut current_dir: String) {
    let text = read_to_string(&mut stream);

    let (method, full_url) = get_method(&text);

    let (path, params) = match method.as_ref() {
        "GET" => parse_url(full_url.split("?").collect::<Vec<&str>>()),
        "POST" | "PUT" => {
            let body = text.split("\r\n\r\n").collect::<Vec<&str>>()[1];
            (
                full_url
                    .split("?")
                    .map(String::from)
                    .collect::<Vec<String>>()[0]
                    .clone(),
                parse_query(body),
            )
        }
        _ => (
            full_url
                .split("?")
                .map(String::from)
                .collect::<Vec<String>>()[0]
                .clone(),
            HashMap::new(),
        ),
    };
    let req = Request::new(&method, params);


    for (url, func) in &router.paths {
        if *url == path {
            let result = func(&req);
            match result {
                Some(html) => {
                    let response = format!("{}{}", create_response_headers("200", "OK"), html);
                    stream.write(response.as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
                _ => {}
            }
            return;
        }
    }



    current_dir.push_str(&path);
    match fs::read(current_dir) {
        Ok(mut content) => {
            let response = format!("{}{}", create_response_headers("200", "OK"), "");
            let mut byte_response = response.as_bytes().to_vec();
            byte_response.append(&mut content);
            stream.write(byte_response.as_slice()).unwrap();
            stream.flush().unwrap();
            return;
        }
        _ => {}
    }


    let page_404 = String::from("<h1>404<h1><h3>Page not found<h3>");
    let html = match router.paths.get("404") {
        Some(func) => {
            let result = func(&req);
            match result {
                Some(html) => html,
                None => page_404,
            }
        }
        None => page_404,
    };

    let response = format!("{}{}", create_response_headers("404", "Not Found"), html);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn read_to_string(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    String::from_utf8_lossy(&buffer[..]).to_string()
}

fn parse_url(url: Vec<&str>) -> (String, HashMap<String, String>) {
    if url.len() == 2 {
        (url[0].to_string(), parse_query(url[1]))
    } else {
        (url[0].to_string(), HashMap::new())
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    query.split("&").for_each(|x| {
        let pair = x.split("=").map(String::from).collect::<Vec<String>>();
        result.insert(pair[0].clone(), pair[1].clone());
    });
    result
}

fn get_method(headers: &str) -> (String, String) {
    let lexs = headers.split_whitespace().collect::<Vec<&str>>();
    (lexs[0].into(), lexs[1].into())
}

fn create_response_headers(code: &str, message: &str) -> String {
    format!("HTTP/1.1 {} {}\r\n\r\n", code, message)
}


#[cfg(test)]
mod tests {
    use super::Server;
    use super::router::Router;
    use super::router::Request;
    use std::collections::HashMap;


    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn creating_server(){
        let server = Server::new("127.0.0.1", "8080");
        assert_eq!(server.ip, "127.0.0.1".to_string());
        assert_eq!(server.port, "8080");
    }


    #[test]
    fn appending_paths(){
        let mut router = Router::new();
        router.add_path("/home", handle);
        let req = Request::new("GET", HashMap::new());
        assert_eq!(1, router.paths.len());
        assert_eq!(&"/home", router.paths.keys().nth(0).unwrap());
        assert_eq!("Hello world".to_string(), router.paths.get("/home").unwrap()(&req).unwrap());
    }

    fn handle(req: &Request) -> Option<String>{
        Some("Hello world".to_string())
    }
}
