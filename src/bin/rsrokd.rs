extern crate clap;
extern crate iron;
extern crate router;
mod string_error;

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

    let mut router = Router::new();
    router.any("/", dummy, "dummy");

    let mut chain = Chain::new(router);
    let rsrokd_middleware = RsrokdMiddleware {
        host: host.to_string(),
    };
    chain.link_around(rsrokd_middleware);
    Iron::new(chain).http(host).unwrap();
}

fn dummy(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::NotFound)))
}

struct RsrokdMiddleware {
    host: String,
}

impl AroundMiddleware for RsrokdMiddleware {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        let new_handler = move |req: &mut Request| -> IronResult<Response> {
            match req.headers.clone().get::<Host>() {
                Some(host) if self.is_root_host(host.hostname.to_owned(), host.port) => {
                    handler.handle(req)
                }
                Some(host) => Ok(Response::with(
                    (status::Ok, ContentType::json().0, host.clone().hostname),
                )),
                _ => Err(IronError::new(
                    StringError("invalid request".to_string()),
                    status::BadRequest,
                )),
            }
        };
        Box::new(new_handler)
    }
}

impl RsrokdMiddleware {
    fn is_root_host(&self, target_host: String, target_port: Option<u16>) -> bool {
        let port = target_port.map_or("".to_owned(), |port| ":".to_string() + &port.to_string());
        self.host == (target_host + &port)
    }
}
