#![feature(proc_macro, plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate r2d2;
extern crate r2d2_diesel;
#[macro_use]
extern crate lazy_static;
//#[macro_use]
//#extern crate log;
//#extern crate env_logger;
extern crate chrono;
extern crate lru_time_cache;
extern crate textnonce;

pub mod schema;
pub mod models;
mod pages;
mod session;

use pages::{blog, auth};

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{
        foreign_links {
            R2D2Error(::r2d2::GetTimeout);
            DieselError(::diesel::result::Error);
        }
    }
}

use dotenv::dotenv;
use errors::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::env;
use r2d2_diesel::ConnectionManager;

use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Request, Outcome, Form, FromRequest};
use rocket::response::{Redirect, Failure};
use rocket_contrib::Template;

use models::*;
// use schema::posts::dsl::*;


// Create a static connection pool
// See: http://neikos.me/Using_Rust_for_Webdev_as_a_Hobby_Programmer.html
lazy_static! {
    static ref CONNECTION: r2d2::Pool<ConnectionManager<PgConnection>> = {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let config = r2d2::Config::default();
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::new(config, manager).expect("Failed to create pool")
    };
}

/// Acquire a connection to the database from the connection pool
fn connection() -> r2d2::Pool<ConnectionManager<PgConnection>> {
    CONNECTION.clone()
}

/// Get all posts for this blog
pub fn get_posts() -> Result<Vec<Post>> {
    use schema::posts::dsl::*;

    let results = posts.filter(is_published.eq(true))
        .load::<Post>(&*try!(connection().get()))?;

    Ok(results)
}

pub fn new_draft<'a>(post_title: &'a str, post_body: &'a str) -> Result<Post> {
    use schema::posts;

    let draft = NewPost::draft(post_title, post_body);

    Ok(diesel::insert(&draft).into(posts::table)
        .get_result(&*try!(connection().get()))?)
}


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[error(403)]
fn forbidden() -> &'static str {
    "Unauthorized!"
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, auth::login])
        .mount("/blog",
               routes![blog::index, blog::new_post, blog::new_post_noauth, blog::new_post_submit])
        .catch(errors![forbidden])
        .launch();
}