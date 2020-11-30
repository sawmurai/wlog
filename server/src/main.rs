#![feature(proc_macro_hygiene, decl_macro)]

use std::sync::Mutex;

use rocket::http::Status;
use rocket::{request::FromRequest, response::content};
use rocket::{Outcome, State};
use rocket_contrib::json::Json;
use serde::Deserialize;
use std::env;
use worklog::Entry;
use worklog::Wlog;

extern crate worklog;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

type WlogMutex = Mutex<Wlog>;

struct ApiKey(String);
#[derive(Debug)]
enum ApiKeyError {
    NotProvided,
    Wrong,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let headers: Vec<_> = request.headers().get("authorization").collect();
        let pk = env::var("SECRET").unwrap();
        for value in headers {
            if value == pk {
                return Outcome::Success(ApiKey(value.to_string()));
            } else {
                return Outcome::Failure((Status::Unauthorized, ApiKeyError::Wrong));
            }
        }

        Outcome::Failure((Status::Unauthorized, ApiKeyError::NotProvided))
    }
}

#[derive(Deserialize)]
struct EntryCollection(Vec<NewEntry>);

#[derive(Deserialize)]
struct NewEntry {
    id: String,
    message: String,
    time_created: String,
}

impl Into<Entry> for NewEntry {
    fn into(self) -> Entry {
        Entry::from_date(&Some(self.id), &self.time_created, &self.message)
    }
}

#[get("/")]
fn index(wlog: State<WlogMutex>, _apikey: ApiKey) -> content::Json<String> {
    if let Ok(wlog) = wlog.lock().as_ref() {
        return content::Json(format!(
            "[{}]",
            wlog.find_all()
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(",")
        ));
    }

    content::Json(String::new())
}

#[post("/", format = "json", data = "<collection>")]
fn post_log(wlog: State<WlogMutex>, collection: Json<EntryCollection>, _apikey: ApiKey) -> Status {
    if let Ok(wlog) = wlog.lock().as_mut() {
        let collection = collection.0;

        for entry in collection.0 {
            wlog.sync(&entry.into());
        }
    }

    Status::Ok
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, post_log])
        .manage(Mutex::new(Wlog::new(
            &env::var("DATABASE").unwrap_or("wlog.sqlite".to_string()),
        )))
        .launch();
}
