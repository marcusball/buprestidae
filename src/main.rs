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
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;

pub mod schema;
pub mod models;

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

use rocket::http::{Cookie, Cookies};
use rocket::request::{Request, Outcome, Form, FromRequest};
use rocket::response::Redirect;
use rocket_contrib::Template;

use models::*;
// use schema::posts::dsl::*;


#[derive(FromForm,Debug)]
struct NewPostForm {
    title: String,
    body: String,
}

struct UserSession();

impl<'a, 'r> FromRequest<'a, 'r> for UserSession {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<UserSession, ()> {
        match request.cookies().find("BUP_SESSION") {
            None => rocket::Outcome::Forward(()),
            Some(cookie) => {
                match cookie.value.as_str() {
                    "supersecretkey" => rocket::Outcome::Success(UserSession()),
                    _ => rocket::Outcome::Forward(()),
                }
            }
        }
    }
}



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

#[get("/")]
fn blog_index() -> Result<Template> {
    #[derive(Serialize)]
    struct BlogIndexContext {
        posts: Vec<Post>,
    }

    let context =
        BlogIndexContext { posts: get_posts().chain_err(|| "Failed to load posts from database")? };

    Ok(Template::render("blog_index", &context))
}

#[get("/new")]
fn blog_new_post(session: UserSession) -> Template {
    #[derive(Serialize)]
    struct NewPostContext {

    }

    let context = NewPostContext {};


    Template::render("blog_new_post", &context)
}

#[get("/new", rank = 2)]
fn blog_new_post_noauth() -> Redirect {
    Redirect::to("/blog")
}

#[post("/new", data="<post>")]
fn blog_new_post_submit(post: Form<NewPostForm>) -> Result<Redirect> {
    use schema::posts;
    let post = post.get();

    let draft = NewPost::new(post.title.as_str(), post.body.as_str());
    diesel::insert(&draft).into(posts::table)
        .get_result::<Post>(&*(connection().get()?))?;
    Ok(Redirect::to("/blog"))
}

#[get("/login")]
fn login(cookies: &Cookies) -> Redirect {
    let mut session = Cookie::new("BUP_SESSION".into(), "supersecretkey".into());
    session.httponly = true;
    cookies.add(session);

    Redirect::to("/")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, login])
        .mount("/blog",
               routes![blog_index, blog_new_post, blog_new_post_noauth, blog_new_post_submit])
        .launch();
}