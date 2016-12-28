use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Request, Outcome, Form, FromRequest};
use rocket::response::{Redirect, Failure};
use rocket_contrib::Template;

use session;
use session::{UserSession, SESSIONS};

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain!{
    foreign_links {
        R2D2Error(::r2d2::GetTimeout);
        DieselError(::diesel::result::Error);
    }
}

#[get("/login")]
fn login(cookies: &Cookies) -> Result<Redirect> {
    let _ = SESSIONS.write()
        .expect("Failed to update session cache")
        .insert("supersecretkey".into(), UserSession());
    let mut session = Cookie::new("BUP_SESSION".into(), "supersecretkey".into());
    session.httponly = true;
    cookies.add(session);

    Ok(Redirect::to("/"))
}