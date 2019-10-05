extern crate argparse;
extern crate chrono;
extern crate dirs;
extern crate notify_rust;
extern crate worklog;

use notify_rust::NotificationHint as Hint;
use notify_rust::Notification;
use notify_rust::*;

use argparse::{ArgumentParser, Store, StoreTrue};

fn main() {
    let mut log = worklog::Wlog::new(&format!("{}/{}", &dirs::data_dir().unwrap().to_str().unwrap(), "wlog.sqlite"));
    
    let mut date = "".to_string();
    let mut message = "".to_string();
    let mut search = "".to_string();
    let mut notification = false;

    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Work log into sqlite");
        ap.refer(&mut message).add_option(&["--message", "-m"], Store, "Message to log");
        ap.refer(&mut search).add_option(&["--search", "-s"], Store, "Message to search for");
        ap.refer(&mut date).add_option(&["--date", "-d"], Store, "Date to read from / write into");
        ap.refer(&mut notification).add_option(&["--notification", "-n"], StoreTrue, "Print output into a desktop notification");
        ap.parse_args_or_exit();
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
        .show().unwrap();
    } else {
        println!("{}", output);
    }
}
