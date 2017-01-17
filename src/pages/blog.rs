use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::*;
use session::UserSession;

use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Request, Outcome, Form, FromRequest, FromParam};
use rocket::response::{Redirect, Failure};
use rocket_contrib::Template;

use slug;

use pulldown_cmark::{html, Parser};

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

impl<'a> FromParam<'a> for Post {
    type Error = self::Error;
    fn from_param(param: &'a str) -> Result<Self> {
        use schema::posts::dsl::*;

        let post_slug: String = String::from_param(param)?;

        let post = posts.filter(slug.eq(post_slug))
            .first::<Post>(&*::connection().get()?)?;

        Ok(post)
    }
}

/// Struct describing a single post, which will be supplied
/// to page templates for rendering the specified post.
#[derive(Serialize)]
struct PostContext {
    title: String,
    body: String,
    url: String,
}

impl PostContext {
    pub fn from_post(post: Post) -> PostContext {
        PostContext {
            url: get_post_url(&post),
            title: post.title,
            body: render_markdown(&post.body),
        }
    }
}


#[get("/")]
pub fn index() -> Result<Template> {
    use schema::posts::dsl::*;

    #[derive(Serialize)]
    struct BlogIndexContext {
        posts: Vec<PostContext>,
    }

    let context = BlogIndexContext {
        posts: posts.filter(is_published.eq(true))
            .order(publish_date.desc())
            .load::<Post>(&*try!(::connection().get()))
            .chain_err(|| "Failed to load posts from database")?
            .into_iter()
            .map(|post| PostContext::from_post(post))
            .collect(),
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

    let post_slug = slug::slugify(&post.title);

    let draft = NewPost::new(post.title.as_str(), post.body.as_str(), post_slug.as_str());
    diesel::insert(&draft).into(posts::table)
        .get_result::<Post>(&*(::connection().get()?))?;
    Ok(Redirect::to("/blog"))
}

#[get("/<post>")]
pub fn display_post(post: Post) -> Result<Template> {
    Ok(Template::render("blog/post", &PostContext::from_post(post)))
}



/// Create a url for linking to the given `post`.
fn get_post_url(post: &Post) -> String {
    format!("/blog/{}", &post.slug)
}

/// Takes a string `body` containing markdown (CommonMark)
/// and returns the rendered HTML of the body.
fn render_markdown<S: AsRef<str>>(body: S) -> String {
    // Create a destination string buffer
    let mut rendered = String::new();

    // Parse the markdown
    let parser = Parser::new(body.as_ref());

    // Iterate over the parsed markdown, placing output in the `rendered` buffer.
    html::push_html(&mut rendered, parser);

    // return the HTML body
    return rendered;
}