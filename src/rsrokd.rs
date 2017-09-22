extern crate clap;
extern crate iron;
extern crate router;
mod string_error;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use clap::{App, Arg};
use iron::*;
use iron::headers::*;
use router::Router;
use string_error::StringError;


fn main() {
    let matches = App::new("rsrokd")
        .version("0.1.0")
        .arg(
            Arg::with_name("host")
                .long("host")
                .value_name("HOST")
                .help("set allowed host")
                .takes_value(true),
        )
        .get_matches();
    println!("host: {}", matches.value_of("host").unwrap());
    let host = matches.value_of("host").unwrap();
    let rsrokd = Rsrokd {
        host: host.to_string(),
        tunnels: Arc::new(Mutex::new(HashMap::new())),
    };
    let mut api_handler = Router::new();
    api_handler.any("/", dummy, "dummy");
    let tunnel_handler = TunnelHandler {};
    let handler = RsrokdHandler {
        host: host.to_string(),
        api_handler: Box::new(api_handler),
        tunnel_handler: Box::new(tunnel_handler),
    };
    Iron::new(handler).http(host).unwrap();
}

struct Tunnel;

struct Rsrokd {
    pub host: String,
    pub tunnels: Arc<Mutex<HashMap<String, Tunnel>>>
}

fn dummy(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::NotFound)))
}
struct RsrokdHandler {
    host: String,
    api_handler: Box<Handler>,
    tunnel_handler: Box<Handler>,
}
impl Handler for RsrokdHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match req.headers.clone().get::<Host>() {
            Some(host) if self.is_root_host(host.hostname.to_owned(), host.port) => {
                self.api_handler.handle(req)
            }
            Some(_) => self.tunnel_handler.handle(req),
            _ => Err(IronError::new(
                StringError("invalid request".to_string()),
                status::BadRequest,
            )),
        }
    }
}

impl RsrokdHandler {
    fn is_root_host(&self, target_host: String, target_port: Option<u16>) -> bool {
        self.host == to_host_str(target_host, target_port)
    }
}

struct TunnelHandler {}

impl Handler for TunnelHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let headers = req.headers.clone();
        let host = headers.get::<Host>().unwrap();
        Ok(Response::with((
            status::Ok,
            ContentType::json().0,
            to_host_str(host.hostname.to_owned(), host.port),
        )))
    }
}

fn to_host_str(hostname: String, port: Option<u16>) -> String {
    let port = port.map_or("".to_owned(), |port| ":".to_string() + &port.to_string());
    hostname + &port
}
