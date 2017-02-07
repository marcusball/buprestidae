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
use time;

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
    struct LoginContext {}
    let context = LoginContext {};

    Template::render("auth/login", &context)
}

#[post("/login", data = "<form>")]
fn login_post(form: Form<LoginForm>, cookies: &Cookies) -> Result<Redirect> {
    use models::User;
    use ::schema::users::dsl::*;

    let form = form.get();

    let user: User = users.filter(email.eq(&form.email))
        .first(&*(::connection().get()?))
        .chain_err(|| ErrorKind::LoginEmailDoesNotExist(form.email.clone()))?;

    // Test if the OTP code provided is valid for this user
    if is_valid_totp_code(&form.code, &user.code, 1) {
        let session_id = SessionStore::new_id();
        SessionStore::insert(session_id.clone(), UserSession::new(user));
        let mut session = Cookie::new("BUP_SESSION", session_id);
        session.set_http_only(true);
        cookies.add(session);

        Ok(Redirect::to("/"))
    } else {
        Err(ErrorKind::InvalidLoginOTP(form.email.clone(), form.code.clone()).into())
    }
}

/// Check whether the given TOTP `code` is valid for the specified
/// base32 `key` value. This will test if the code is correct for
/// the code at the current time value, as well as the codes at ±N times
/// in the past and future, where N is the `plusminus` value.
fn is_valid_totp_code(code: &String, key: &String, plusminus: u8) -> bool {
    // The timestep, in seconds.
    let period = 30;

    let current_time = time::now().to_timespec().sec;

    // Create a list of period offsets for which to test if the code is valid
    let offset_list = plus_or_minus_offset_list(plusminus);

    offset_list.iter().any(|offset| {
        TOTPBuilder::new()
            .period(period)
            .base32_key(key)
            .timestamp(current_time + (period as i64 * *offset as i64))
            .finalize()
            .and_then(|totp| Ok(totp.is_valid(code)))
            .unwrap_or(false)
    })
}

/// Generate a vector of integers from `-plusminus` to `plusminus`.
///
/// # Examples
///
/// ```
/// let offset_list = plus_or_minus_offset_list(2);
/// assert_eq!(offset_list, vec![0, -1, 1, -2, 2]);
/// ```
fn plus_or_minus_offset_list(plusminus: u8) -> Vec<i16> {
    let mut offset_list: Vec<i16> = vec![0];
    for offset in 1..(plusminus + 1) {
        offset_list.push(offset as i16 * -1);
        offset_list.push(offset as i16);
    }
    offset_list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plus_or_minus_offset_list_works() {
        assert_eq!(plus_or_minus_offset_list(0), vec![0]);
        assert_eq!(plus_or_minus_offset_list(1), vec![0, -1, 1]);
        assert_eq!(plus_or_minus_offset_list(2), vec![0, -1, 1, -2, 2]);
        assert_eq!(plus_or_minus_offset_list(3), vec![0, -1, 1, -2, 2, -3, 3]);
    }
}