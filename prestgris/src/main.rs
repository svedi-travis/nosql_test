extern crate iron;
extern crate mount;
extern crate persistent;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_redis;
extern crate r2d2_postgres;
extern crate rand;
extern crate redis;
#[macro_use]
extern crate router;

use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use mount::Mount;
use postgres::Connection;
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use r2d2_redis::RedisConnectionManager;
use rand::{thread_rng, Rng};
use redis::{Commands, Value};
use router::Router;
use std::default::Default;
use std::fmt::Write;
use std::fs::File;
use std::io::prelude::*;
use std::thread;

struct MachineCode {
  id: i32,
  value: String,
}

struct AppDb;
impl Key for AppDb { type Value = (PostgresPool, RedisPool); }

type PostgresPool = Pool<PostgresConnectionManager>;
type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;
type RedisPool = Pool<RedisConnectionManager>;
type RedisPooledConnection = PooledConnection<RedisConnectionManager>;

static POSTGRES_URI: &'static str = "postgres://rust:databasesystemsthecompletebook@10.0.0.4";
static REDIS_URI: &'static str = "redis://10.0.0.4";


fn main() {
  // Set up database
  let pmanager = r2d2_postgres::PostgresConnectionManager::new
                   ( POSTGRES_URI
                   , r2d2_postgres::TlsMode::None
                   ).unwrap();
  let pconfig = r2d2::Config::builder().pool_size(32).build();
  let ppool = r2d2::Pool::new(pconfig, pmanager).unwrap();
  let pconn = ppool.get().unwrap();

  println!("Postgres connected!");

  let rmanager = r2d2_redis::RedisConnectionManager::new(REDIS_URI).unwrap();
  let rconfig = r2d2::Config::builder().pool_size(32).build();
  let rpool = r2d2::Pool::new(rconfig, rmanager).unwrap();
  let rconn = rpool.get().unwrap();

  println!("Redis connected!");

  pconn.execute("DROP TABLE IF EXISTS machine_code", &[]).unwrap();
  pconn.execute( "CREATE TABLE machine_code (
                    id		SERIAL PRIMARY KEY,
                    value	VARCHAR NOT NULL
                 )"
               , &[]).unwrap();
  pconn.execute("CREATE INDEX ON machine_code (value)", &[]).unwrap();
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
  middleware.link(persistent::Read::<AppDb>::both((ppool, rpool)));

  println!("Starting server on port 8080.");

  Iron::new(middleware).http("0.0.0.0:8080").unwrap();
}

fn token_handler(req: &mut Request) -> IronResult<Response> {
  let pools = req.get::<persistent::Read<AppDb>>().unwrap();
  let pconn = pools.0.get().unwrap();
  let rconn = pools.1.get().unwrap();
  let router = req.extensions.get::<Router>().unwrap();

  let token = router.find("token").unwrap();

  let rows = pconn.query(
                 "SELECT COUNT(*) FROM machine_code WHERE value = $1"
               , &[&token]
             ).unwrap();

  if rows.len() > 0 && rows.get(0).get::<usize,i64>(0) > 0 {
    return Ok(Response::with((status::Ok, "Welcome back!")));
  }

  let count: u64 = rconn.incr("count", 1u64).unwrap();
  if count >= 15000000000 {
    let _: () = rconn.incr("count", -1i64).unwrap();
    return Ok(Response::with((status::Ok, "Limit exceeded! Sorry!")));
  }

  let rows = pconn.execute(
               "INSERT INTO machine_code (value) VALUES ($1)"
            , &[&token]).unwrap();
  if rows == 0 { let _: () = rconn.incr("count", -1i64).unwrap(); }
  Ok(Response::with((status::Ok, "New cusomters are always welcome!")))
}
