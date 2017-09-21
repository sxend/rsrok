extern crate clap;
extern crate iron;
extern crate router;
mod string_error;

use clap::{Arg, App};
use iron::*;
use iron::headers::*;
use router::Router;
use string_error::StringError;

fn main() {
    let matches = App::new("rsrokd")
        .version("0.1.0")
        .arg(Arg::with_name("host")
            .long("host")
            .value_name("HOST")
            .help("set allowed host")
            .takes_value(true))
        .get_matches();
    matches.value_of("host");
    println!("host: {}", matches.value_of("host").unwrap());
    let host = matches.value_of("host").unwrap();
    let mut router = Router::new();
    router.get("/api/v1/join", join, "GET /api/v1/join");
    let mut chain = Chain::new(router);
    chain.link_around(rsrokd_middleware);
    Iron::new(chain).http(host).unwrap();
}

fn join(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, ContentType::json().0, "join")))
}
fn rsrokd_middleware(handler: Box<Handler>) -> Box<Handler> {
    let new_handler = move |req: &mut Request| -> IronResult<Response> {
        match req.headers.clone().get::<Host>() {
            Some(host) if host.hostname == "00.rsrokd.local" =>
                Ok(Response::with((status::Ok, ContentType::json().0, host.clone().hostname))),
            Some(host) if host.hostname == "rsrokd.local" => {
                let req = req;
                handler.handle(req)
            }
            _ => Err(IronError::new(
                StringError("invalid request".to_string()),
                status::BadRequest,
            ))
        }
    };
    Box::new(new_handler)
}

