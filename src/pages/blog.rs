use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::*;
use session::UserSession;

use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Request, Outcome, Form, FromRequest};
use rocket::response::{Redirect, Failure};
use rocket_contrib::Template;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain!{
    foreign_links {
        R2D2Error(::r2d2::GetTimeout);
        DieselError(::diesel::result::Error);
    }
}


#[derive(FromForm,Debug)]
pub struct NewPostForm {
    title: String,
    body: String,
}


#[get("/")]
pub fn index() -> Result<Template> {
    #[derive(Serialize)]
    struct BlogIndexContext {
        posts: Vec<Post>,
    }

    let context = BlogIndexContext {
        posts: ::get_posts().chain_err(|| "Failed to load posts from database")?,
    };

    Ok(Template::render("blog/index", &context))
}

#[get("/new")]
pub fn new_post(session: UserSession) -> Template {
    #[derive(Serialize)]
    struct NewPostContext {

    }

    let context = NewPostContext {};


    Template::render("blog/new_post", &context)
}

#[get("/new", rank = 2)]
pub fn new_post_noauth() -> Failure {
    Failure(Status::Forbidden)
}

#[post("/new", data="<post>")]
pub fn new_post_submit(post: Form<NewPostForm>) -> Result<Redirect> {
    use schema::posts;
    let post = post.get();

    let draft = NewPost::new(post.title.as_str(), post.body.as_str());
    diesel::insert(&draft).into(posts::table)
        .get_result::<Post>(&*(::connection().get()?))?;
    Ok(Redirect::to("/blog"))
}

#[get("/<post_id>")]
pub fn display_post(post_id: i32) -> Result<Template> {
    use schema::posts::dsl::*;

    let post = posts.find(post_id)
        .first::<Post>(&*::connection().get()?)?;

    Ok(Template::render("blog/post", &post))
}