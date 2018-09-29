extern crate argparse;
extern crate rusqlite;
extern crate chrono;
extern crate dirs;
extern crate notify_rust;

use notify_rust::NotificationHint as Hint;
use notify_rust::Notification;
use notify_rust::*;

use rusqlite::Connection;
use argparse::{ArgumentParser, Store, StoreTrue};
use chrono::prelude::*;

#[derive(Debug)]
struct Entry {
    message: String,
    time_created: String
}

fn main() {
    let dt = Local::now();
    let mut message = "".to_string();
    let mut date = dt.format("%Y-%m-%d").to_string();
    let mut notification = false;

    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Work log into sqlite");
        ap.refer(&mut message).add_option(&["--message", "-m"], Store, "Message to log");
        ap.refer(&mut date).add_option(&["--date", "-d"], Store, "Date to read from / write into");
        ap.refer(&mut notification).add_option(&["--notification", "-n"], StoreTrue, "Print output into a desktop notification");
        ap.parse_args_or_exit();
    }

    let path = format!("{}/{}", &dirs::data_dir().unwrap().to_str().unwrap(), "wlog.sqlite");
    let conn = Connection::open(&path).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS entry (
                  message         TEXT NOT NULL,
                  time_created    TEXT NOT NULL
                  )", &[]).unwrap();

    if message != "" {
        let entry = Entry {
            message: message,
            time_created: date.to_string()
        };
        conn.execute("INSERT INTO entry (message, time_created)
                    VALUES (?1, ?2)",
                    &[&entry.message, &entry.time_created]).unwrap();
    }

    let mut stmt = conn.prepare("SELECT message, time_created FROM entry WHERE time_created = ?").unwrap();
    let entry_iter = stmt.query_map(&[&date], |row| {
        Entry {
            message: row.get(0),
            time_created: row.get(1)
        }
    }).unwrap();

    let mut output = String::new();

    for entry in entry_iter {
        let e = entry.unwrap();
        let date = e.time_created;
        let message = e.message;

        output = output + &format!("{} - {}\n", &date, &message);
    }

    if notification {
         Notification::new()
        .summary(&format!("Work log from {}", &date))
        .body(&output)
        .icon("dialog-warning")
        .hint(Hint::Resident(true))
        .timeout(Timeout::Never)
        .show().unwrap();
    } else {
        println!("{}", output);
    }
}
