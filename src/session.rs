use std::sync::RwLock;
use lru_time_cache::LruCache;
use textnonce::TextNonce;

use rocket;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Request, Outcome, Form, FromRequest};
use rocket::response::{Redirect, Failure};
use rocket_contrib::Template;

use models::User;


lazy_static! {
    pub static ref SESSIONS: RwLock<LruCache<String, UserSession>> = RwLock::new(LruCache::<String, UserSession>::with_capacity(10));
}


#[derive(Clone)]
pub struct UserSession(User);

impl UserSession {
    pub fn new(user: User) -> UserSession {
        UserSession(user)
    }
}

pub struct SessionStore {}

impl SessionStore {
    pub fn new_id() -> String {
        TextNonce::sized_urlsafe(64).expect("Failed to create textnonce").into_string()
    }

    /// Save the given session to the session store
    pub fn insert(id: String, session: UserSession) {
        let _ = SESSIONS.write()
            .expect("Failed to update session cache")
            .insert(id, session);
    }

    /// Get the session corresponding to the given session `id`, if one exists.
    pub fn get(id: &String) -> Option<UserSession> {
        if let Ok(mut sessions) = SESSIONS.write() {
            return sessions.get(id).map(|session| (*session).clone());
        }
        None
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserSession {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<UserSession, ()> {
        match request.cookies().find("BUP_SESSION") {
            None => rocket::Outcome::Forward(()),
            Some(cookie) => {
                match SessionStore::get(&cookie.value().into()) {
                    Some(session) => {
                        info!("Found session for {} ({})", &session.0.name, &session.0.id);
                        rocket::Outcome::Success(session)
                    }
                    _ => rocket::Outcome::Forward(()),
                }
            }
        }
    }
}
