extern crate iron;
extern crate mount;
extern crate persistent;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rand;
#[macro_use]
extern crate router;

use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use mount::Mount;
use postgres::Connection;
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use rand::{thread_rng, Rng};
use router::Router;
use std::fs::File;
use std::io::prelude::*;

struct Token {
  id: i32,
  value: String,
  count: u32,
}

struct AppDb;
impl Key for AppDb { type Value = PostgresPool; }

type PostgresPool = Pool<PostgresConnectionManager>;
type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;

static DATABASE_URI: &'static str = "postgres://rust:databasesystemsthecompletebook@localhost";

fn main() {
  // Set up database
  let manager = r2d2_postgres::PostgresConnectionManager::new
                  ( DATABASE_URI
                  , r2d2_postgres::TlsMode::None
                  ).unwrap();
  let config = r2d2::Config::builder().pool_size(32).build();
  let pool = r2d2::Pool::new(config, manager).unwrap();
  let conn = pool.get().unwrap();

  conn.execute("DROP TABLE IF EXISTS tokens", &[]).unwrap();
  conn.execute( "CREATE TABLE tokens (
                   id		SERIAL PRIMARY KEY,
                   value	VARCHAR NOT NULL,
                   count        INTEGER
                )"
              , &[]).unwrap();
//
//  for i in 0..100000 {
//    let token = Token {
//        id: i
//      , value: thread_rng().gen_ascii_chars().take(10).collect()
//    };
//
//    conn.execute( "INSERT INTO tokens (id, value) VALUES ($1, $2)"
//                , &[&token.id, &token.value]).unwrap();
//  }
//
//  {
//    let mut file = File::create("rust.urls").unwrap();
//
//    for row in &conn.query("SELECT id, value FROM tokens", &[]).unwrap() {
//      let token = Token {
//        id: row.get(0),
//        value: row.get(1),
//      };
//      file.write_all(b"http://52.166.242.54/").unwrap();
//      file.write_all(&token.value.into_bytes()).unwrap();
//      file.write_all(b"\n").unwrap();
//    }
//  }

  // Start webserver
  let mut mount = Mount::new();

  mount.mount("/", router!(
    token_handler: get "/:token" => token_handler
  ));

  let mut middleware = Chain::new(mount);
  middleware.link(persistent::Read::<AppDb>::both(pool));

  println!("Starting server on port 8080.");

  Iron::new(middleware).http("0.0.0.0:8080").unwrap();
}

fn token_handler(req: &mut Request) -> IronResult<Response> {
  let pool = req.get::<persistent::Read<AppDb>>().unwrap();
  let conn = pool.get().unwrap();
  let router = req.extensions.get::<Router>().unwrap();

  let token = router.find("token").unwrap();

  let modified = conn.execute( "UPDATE tokens SET count = count + 1 WHERE value = $1"
                             , &[&token]).unwrap();
  let valid_token = modified > 0;
  if valid_token {
    Ok(
      Response::with((status::Ok, "Welcome back!"))
    )
  } else {
    conn.execute( "INSERT INTO tokens (value, count) VALUES ($1, $2)"
                , &[&token, &0]).unwrap();
    Ok(
      Response::with((status::Ok, "New customers are always welcome!"))
    )
  }

}
