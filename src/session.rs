use std::sync::RwLock;
use lru_time_cache::LruCache;

use rocket;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Request, Outcome, Form, FromRequest};
use rocket::response::{Redirect, Failure};
use rocket_contrib::Template;


lazy_static! {
    pub static ref SESSIONS: RwLock<LruCache<String, UserSession>> = RwLock::new(LruCache::<String, UserSession>::with_capacity(10));
}


#[derive(Clone)]
pub struct UserSession();


impl<'a, 'r> FromRequest<'a, 'r> for UserSession {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<UserSession, ()> {
        match request.cookies().find("BUP_SESSION") {
            None => rocket::Outcome::Forward(()),
            Some(cookie) => {
                match SESSIONS.write()
                    .expect("Failed to read session cache")
                    .get(&cookie.value) {
                    Some(session) => rocket::Outcome::Success(session.clone()),
                    _ => rocket::Outcome::Forward(()),
                }
            }
        }
    }
}
