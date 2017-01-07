use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Request, Outcome, Form, FromRequest};
use rocket::response::{Redirect, Failure};
use rocket_contrib::Template;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use session;
use session::{UserSession, SessionStore};
use libreauth::oath::TOTPBuilder;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain!{
    errors {
        LoginEmailDoesNotExist(email: String) {
            description("Attempt to login with an email that does not exist")
            display("Invalid login email: '{}'", email)
        }

        InvalidLoginOTP(email: String, code: String) {
            description("Attempt to login with invalid OTP code")
            display("Invalid login combination: '{}', {}", email, code)
        }

        LibreAuthError(e: ::libreauth::oath::ErrorCode) {
            display("OTP code error: {:?}", e)
        }
    }

    foreign_links {
        R2D2Error(::r2d2::GetTimeout);
        DieselError(::diesel::result::Error);
    }
}

#[derive(FromForm,Debug)]
pub struct LoginForm {
    /// Login email address
    email: String,

    /// 2FA authentication code
    code: String,
}

#[get("/login")]
fn login_get() -> Template {
    #[derive(Serialize)]
    struct LoginContext { }
    let context = LoginContext {};

    Template::render("auth/login", &context)
}

#[post("/login", data = "<form>")]
fn login_post(form: Form<LoginForm>, cookies: &Cookies) -> Result<Redirect> {
    use ::models::User;
    use ::schema::users::dsl::*;

    let form = form.get();

    let user: User = users.filter(email.eq(&form.email))
        .first(&*(::connection().get()?))
        .chain_err(|| ErrorKind::LoginEmailDoesNotExist(form.email.clone()))?;

    let is_code_valid = TOTPBuilder::new()
        .base32_key(&user.code)
        .finalize()
        .map_err(|e| Error::from_kind(ErrorKind::LibreAuthError(e)))?
        .is_valid(&form.code);

    if is_code_valid {
        let session_id = SessionStore::new_id();
        SessionStore::insert(session_id.clone(), UserSession::new(user));
        let mut session = Cookie::new("BUP_SESSION".into(), session_id);
        session.httponly = true;
        cookies.add(session);

        Ok(Redirect::to("/"))
    } else {
        Err(ErrorKind::InvalidLoginOTP(form.email.clone(), form.code.clone()).into())
    }
}