use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedBody;
use regex::Regex;
use bcrypt::verify;
use rustc_serialize::json;
use ::utils::*;
use super::session;
use ::users;

pub fn register(req: &mut Request) -> IronResult<Response> {
    let username;
    let email;
    let name;
    let password;
    {
        let form = iexpect!(req.get_ref::<UrlEncodedBody>().ok(), error("form", "Please provide form data!"));
        username = iexpect!(form_field(form, "username"), error("username", "Username is required"));
        email = iexpect!(form_field(form, "email"), error("email", "Email is required"));
        name = iexpect!(form_field(form, "name"), error("name", "Name is required"));
        password = iexpect!(form_field(form, "password"), error("password", "Password is required"));
    }

    let conn = establish_connection();
    let old_user = users::get_by_username(&conn, &username);
    if old_user.is_some() {
        return Ok(Response::with(error("username", "Username taken")));
    }

    let email_regex = Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,4}$").unwrap();
    if !email_regex.is_match(&email) {
        return Ok(Response::with(error("email", "Invalid email")));
    }

    if password.len() < 5 {
        return Ok(Response::with(error("password", "Password too short!")));
    }

    let _user = users::create_user(&conn, &username, email, name, password);
    try!(session::set_username(req, username));

    Ok(Response::with(success()))
}

pub fn login(req: &mut Request) -> IronResult<Response> {
    let username;
    let password;
    {
        let form = iexpect!(req.get_ref::<UrlEncodedBody>().ok(), error("form", "Please provide form data!"));
        username = iexpect!(form_field(form, "username"), error("username", "Username is required"));
        password = iexpect!(form_field(form, "password"), error("password", "Password is required"));
    }

    let conn = establish_connection();
    let user = users::get_by_username(&conn, &username);
    if user.is_none() {
        return Ok(Response::with(error("username", "User doesn't exist")));
    }
    let user = user.unwrap();

    let valid = verify(&password, &user.password).unwrap();

    if valid {
        try!(session::set_username(req, user.username));
        Ok(Response::with(success()))
    } else {
        Ok(Response::with(error("password", "Incorrect password")))
    }
}

pub fn logout(req: &mut Request) -> IronResult<Response> {
    try!(session::delete_username(req));
    Ok(Response::with((status::Ok, success())))
}

pub fn get_user(req: &mut Request) -> IronResult<Response> {
    let username = try!(session::get_username(req));
    let username = iexpect!(username, error("auth", "Not loggede in!"));

    let conn = establish_connection();
    let user = users::get_by_username(&conn, &username.to_string());

    if user.is_none() {
        return Ok(Response::with(error("auth", "Not logged in as existing user!")));
    }

    let user = json::encode(&user.unwrap()).unwrap();
    Ok(Response::with(json(user)))
}

pub fn update_user(req: &mut Request) -> IronResult<Response> {
    let username = try!(session::get_username(req));
    let username = iexpect!(username, error("auth", "Not loggede in!"));

    let conn = establish_connection();
    let old_user = users::get_by_username(&conn, &username.to_string());

    if old_user.is_none() {
        return Ok(Response::with(error("auth", "Not logged in as existing user!")));
    }

    let old_user = old_user.unwrap();

    let username;
    let email;
    let name;
    let password;
    {
        let form = iexpect!(req.get_ref::<UrlEncodedBody>().ok(), error("form", "Please provide form data!"));
        username = iexpect!(form_field(form, "username"), error("username", "Username is required"));
        email = iexpect!(form_field(form, "email"), error("email", "Email is required"));
        name = iexpect!(form_field(form, "name"), error("name", "Name is required"));
        password = form_field(form, "password").unwrap_or("".to_owned());
    }

    let existing_user = users::get_by_username(&conn, &username);
    if old_user.username != username && existing_user.is_some() {
        return Ok(Response::with(error("username", "Username taken")));
    }

    let email_regex = Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,4}$").unwrap();
    if !email_regex.is_match(&email) {
        return Ok(Response::with(error("email", "Invalid email")));
    }

    if !password.is_empty() && password.len() < 5 {
        return Ok(Response::with(error("password", "Password too short!")));
    }

    let user = users::update_user(&conn, old_user, username, email, name, password).unwrap();
    try!(session::set_username(req, user.username));

    Ok(Response::with(success()))
}

pub fn delete_user(req: &mut Request) -> IronResult<Response> {
    let username = try!(session::get_username(req));
    let username = iexpect!(username, error("auth", "Not loggede in!"));

    let conn = establish_connection();
    let old_user = users::get_by_username(&conn, &username.to_string());
    if old_user.is_none() {
        return Ok(Response::with(error("auth", "Not logged in as existing user!")));
    }

    let _user = users::delete_user(&conn, username.to_string());
    try!(session::delete_username(req));

    Ok(Response::with(success()))
}
