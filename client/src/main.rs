extern crate argparse;
extern crate chrono;
extern crate dirs;
extern crate notify_rust;
extern crate worklog;

use argparse::{ArgumentParser, Store, StoreTrue};
use chrono::Local;
use notify_rust::Notification;
use notify_rust::NotificationHint as Hint;
use notify_rust::*;
use reqwest::blocking::Client;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut log = worklog::Wlog::new(&format!(
        "{}/{}",
        &dirs::data_dir().unwrap().to_str().unwrap(),
        "wlog.sqlite"
    ));

    let dt = Local::now();
    let mut date = dt.format("%Y-%m-%d").to_string();
    let mut message = "".to_string();
    let mut search = "".to_string();
    let mut notification = false;
    let mut remote = "".to_string();
    let api_key = std::env::var("API_KEY").unwrap_or(".".to_string());

    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Work log into sqlite");
        ap.refer(&mut message)
            .add_option(&["--message", "-m"], Store, "Message to log");
        ap.refer(&mut search)
            .add_option(&["--search", "-s"], Store, "Message to search for");
        ap.refer(&mut date)
            .add_option(&["--date", "-d"], Store, "Date to read from / write into");
        ap.refer(&mut remote).add_option(
            &["--remote", "-r"],
            Store,
            "Connection details for remote wlog server",
        );
        ap.refer(&mut notification).add_option(
            &["--notification", "-n"],
            StoreTrue,
            "Print output into a desktop notification",
        );
        ap.parse_args_or_exit();
    }

    if remote != "" {
        Client::builder()
            .build()?
            .post(&remote)
            .body(format!(
                "[{}]",
                log.find_all()
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(",")
            ))
            .header("Content-type", "application/json")
            .header("Authorization", api_key.clone())
            .send()?;

        let request = Client::builder()
            .build()?
            .get(&remote)
            .header("Authorization", api_key);

        let resp = request.send()?;

        for entry in resp.json::<Vec<HashMap<String, String>>>()? {
            let id = if let Some(id) = entry.get("id") {
                id.clone()
            } else {
                continue;
            };
            let message = if let Some(message) = entry.get("message") {
                message
            } else {
                continue;
            };
            let time_created = if let Some(time_created) = entry.get("time_created") {
                time_created
            } else {
                continue;
            };

            if log.sync(&worklog::Entry::from_date(
                &Some(id.clone()),
                time_created,
                message,
            )) {
                eprintln!("Imported {}", id);
            }
        }

        return Ok(());
    }

    if message != "" {
        log.log(&worklog::Entry::now(&message));
    }

    let title;
    let results;

    if search == "" {
        results = log.find_by_date(&date);
        title = format!("Work log from {}", &date);
    } else {
        results = log.find_by_message(&message);
        title = format!("Work log like {}", &search);
    }

    let mut output = String::new();

    for e in results {
        let date = e.time_created;
        let message = e.message.trim().to_string();

        output.push_str(&format!("{} - {}\n", &date, &message));
    }

    if notification {
        Notification::new()
            .summary(&title)
            .body(&output)
            .icon("dialog-warning")
            .hint(Hint::Resident(true))
            .timeout(Timeout::Never)
            .show()
            .unwrap();
    } else {
        println!("{}", output);
    }

    Ok(())
}
