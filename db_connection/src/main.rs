extern crate iron;
extern crate mount;
extern crate postgres;
extern crate rand;
#[macro_use]
extern crate router;

use iron::prelude::*;
use iron::status;
use mount::Mount;
use postgres::Connection;
use rand::{thread_rng, Rng};
use router::Router;

struct Token {
  id: i32,
  value: String,
}

static DATABASE_URI: &'static str = "postgres://rust:databasesystemsthecompletebook@localhost";

fn main() {
  // Set up database
  let conn = Connection::connect( DATABASE_URI
                                , postgres::TlsMode::None
                                ).unwrap();

  conn.execute("DROP TABLE IF EXISTS tokens", &[]).unwrap();
  conn.execute( "CREATE TABLE tokens (
                   id		SERIAL PRIMARY KEY,
                   value	VARCHAR NOT NULL
                )"
              , &[]).unwrap();

  for i in (0..100000) {
    let token = Token {
        id: i
      , value: thread_rng().gen_ascii_chars().take(10).collect()
    };

    conn.execute( "INSERT INTO tokens (id, value) VALUES ($1, $2)"
                , &[&token.id, &token.value]).unwrap();
  }


  for row in &conn.query("SELECT id, value FROM tokens", &[]).unwrap() {
    let token = Token {
      id: row.get(0),
      value: row.get(1),
    };
    println!("{}", token.value);
  }

  // Start webserver
  let mut mount = Mount::new();

  mount.mount("/", router!(
    token_handler: get "/:token" => token_handler
  ));

  println!("Starting server on port 8080.");

  Iron::new(mount).http("0.0.0.0:8080").unwrap();
}

fn token_handler(req: &mut Request) -> IronResult<Response> {
  let conn = Connection::connect( DATABASE_URI
                                , postgres::TlsMode::None
                                ).unwrap();
  let router = req.extensions.get::<Router>().unwrap();

  let token = router.find("token").unwrap();

  for row in &conn.query("SELECT id FROM tokens WHERE value = $1", &[&token]).unwrap() {
    let id: i32 = row.get(0);
    println!("{}", id);
  }

  Ok(
    Response::with((status::Ok, token))
  )
}
