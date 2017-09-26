extern crate clap;
extern crate iron;
extern crate router;
extern crate uuid;
mod string_error;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use clap::{App, Arg};
use iron::*;
use iron::headers::*;
use router::Router;
use string_error::StringError;
use uuid::Uuid;

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
    let handler = RsrokdHandler::new(rsrokd);
    Iron::new(handler).http(host).unwrap();
}

#[derive(Debug)]
struct Tunnel {
    pub id: String,
}

#[derive(Debug)]
struct Rsrokd {
    pub host: String,
    pub tunnels: Arc<Mutex<HashMap<String, Tunnel>>>,
}

struct RsrokdHandler {
    rsrokd: Arc<Rsrokd>,
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
    fn new(rsrokd: Rsrokd) -> RsrokdHandler {
        let rsrokd = Arc::new(rsrokd);
        let api_handler = ApiHandler::new(rsrokd.clone());
        let tunnel_handler = TunnelHandler::new(rsrokd.clone());
        RsrokdHandler {
            rsrokd: rsrokd.clone(),
            api_handler: Box::new(api_handler),
            tunnel_handler: Box::new(tunnel_handler),
        }
    }
    fn is_root_host(&self, target_host: String, target_port: Option<u16>) -> bool {
        self.rsrokd.host == to_host_str(target_host, target_port)
    }
}

struct ApiHandler {
    rsrokd: Arc<Rsrokd>,
    router: Router,
}

impl ApiHandler {
    fn new(rsrokd: Arc<Rsrokd>) -> ApiHandler {
        let mut router = Router::new();
        router.any("/", ApiHandler::dummy, "dummy");
        router.get(
            "/api/v1/join",
            ApiHandler::join_route(rsrokd.clone()),
            "join",
        );
        ApiHandler {
            rsrokd: rsrokd,
            router: router,
        }
    }
    fn dummy(req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::NotFound)))
    }
    fn join_route(rsrokd: Arc<Rsrokd>) -> Box<Handler> {
        let foo = move |req: &mut Request| {
            let rsrokd = rsrokd.clone();
            let mut tunnels = rsrokd.tunnels.lock().unwrap();
            let id = Uuid::new_v4().hyphenated().to_string();
            let tunnel = tunnels.entry(id.to_owned()).or_insert(Tunnel {
                id: id.to_owned(), // register ws channels
            });
            Ok(Response::with((
                status::Created,
                ContentType::json().0,
                format!("{:?}", tunnel),
            )))
        };
        Box::new(foo)
    }
}
impl Handler for ApiHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        println!(
            "execute api. current tunnels len: {}",
            self.rsrokd.tunnels.lock().unwrap().len()
        );
        self.router.handle(req)
    }
}

struct TunnelHandler {
    rsrokd: Arc<Rsrokd>,
}

impl TunnelHandler {
    fn new(rsrokd: Arc<Rsrokd>) -> TunnelHandler {
        TunnelHandler { rsrokd: rsrokd }
    }
}

impl Handler for TunnelHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        println!(
            "execute tunnel. current tunnels len: {}",
            self.rsrokd.tunnels.lock().unwrap().len()
        );
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
