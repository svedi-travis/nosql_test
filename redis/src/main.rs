extern crate iron;
extern crate mount;
extern crate persistent;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate redis;
#[macro_use]
extern crate router;

use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use mount::Mount;
use r2d2::{Pool, PooledConnection};
use r2d2_redis::RedisConnectionManager;
use rand::{thread_rng, Rng};
use redis::{Commands, Value};
use router::Router;
use std::default::Default;
use std::fmt::Write;
use std::fs::File;
use std::io::prelude::*;
use std::thread;

struct Token {
  id: i32,
  value: String,
  count: u32,
}

struct AppDb;
impl Key for AppDb { type Value = RedisPool; }

type RedisPool = Pool<RedisConnectionManager>;
type RedisPooledConnection = PooledConnection<RedisConnectionManager>;

fn main() {
  // Set up database
  let manager = r2d2_redis::RedisConnectionManager::new("redis://localhost").unwrap();
  let config = r2d2::Config::builder().pool_size(32).build();
  let pool = r2d2::Pool::new(config, manager).unwrap();
  let conn = pool.get().unwrap();

//  conn.execute("DROP TABLE IF EXISTS tokens", &[]).unwrap();
//  conn.execute( "CREATE TABLE tokens (
//                   id		SERIAL PRIMARY KEY,
//                   value	VARCHAR NOT NULL,
//                   count        INTEGER
//                )"
//              , &[]).unwrap();
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

  let member: bool = conn.sismember("machine_codes", token).unwrap();
  if member {
    return Ok(Response::with((status::Ok, "Welcome back!")));
  }

  let count: u64 = conn.incr("count", 1u64).unwrap();
  if count >= 15000000000 {
    let _: () = conn.incr("count", -1i64).unwrap();
    return Ok(Response::with((status::Ok, "Limit exceeded! Sorry!")));
  }

  let _: () = conn.sadd("machine_codes", token).unwrap();
  Ok(Response::with((status::Ok, "New cusomters are always welcome!")))
}

// Old version that is abit different
//  let mut limit_key = String::new();
//  write!(&mut limit_key, "key:{}:limit", token);
//
//  let mut count_key  = String::new();
//  write!(&mut count_key , "key:{}:count", token);
//
//  let limit: Option<u64> = conn.get(&limit_key).unwrap();
//
//  if limit.is_none() {
//    let _: () = conn.set(&limit_key, 10u64).unwrap();
//    let _: () = conn.set(&count_key,  0u64).unwrap();
//    return Ok(Response::with((status::Ok, "New customers are always welcome!")));
//  }
//
//  let limit = limit.unwrap();
//  let count: u64 = conn.incr(&count_key, 1u64).unwrap();
//
//  if count < limit {
//    return Ok(Response::with((status::Ok, "Welcome back!")));
//  } else {
//    let count: u64 = conn.incr(&count_key, -1i64).unwrap();
//    return Ok(Response::with((status::Ok, "Limit exceeded!")));
//  }
