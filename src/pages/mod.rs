pub mod blog;
pub mod auth;


use rocket_contrib::Template;


#[get("/")]
pub fn index() -> Template {
    #[derive(Serialize)]
    struct EmptyContext {};

    let context = EmptyContext {};

    Template::render("index", &context)
}