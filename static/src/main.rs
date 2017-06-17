extern crate iron;
extern crate mount;
#[macro_use]
extern crate router;

use iron::prelude::*;
use iron::status;
use mount::Mount;
use router::Router;

fn main() {
  let mut mount = Mount::new();

  mount.mount("/", router!(
    handler_1: get "/1/:value" => handler_1,
    handler_2: get "/2/:value" => handler_2,
    handler_3: get "/3/:value" => handler_3,
    handler_4: get "/4/:value" => handler_4,
  ));

  println!("Starting server on port 8080.");

  Iron::new(mount).http("0.0.0.0:8080").unwrap();
}

fn handler_1(req: &mut Request) -> IronResult<Response> {
  let router = req.extensions.get::<Router>().unwrap();

  let token = router.find("value").unwrap();

  Ok(
    Response::with((status::Ok, "HELLO!!!!"))
  )
}

fn handler_2(req: &mut Request) -> IronResult<Response> {
  let router = req.extensions.get::<Router>().unwrap();

  let token = router.find("value").unwrap();

  Ok(
    Response::with((status::Ok, "HELLO!!!!"))
  )
}

fn handler_3(req: &mut Request) -> IronResult<Response> {
  let router = req.extensions.get::<Router>().unwrap();

  let token = router.find("value").unwrap();

  Ok(
    Response::with((status::Ok, "HELLO!!!!"))
  )
}

fn handler_4(req: &mut Request) -> IronResult<Response> {
  let router = req.extensions.get::<Router>().unwrap();

  let token = router.find("value").unwrap();

  Ok(
    Response::with((status::Ok, "HELLO!!!!"))
  )
}
