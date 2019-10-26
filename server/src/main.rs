extern crate argparse;
extern crate futures;
extern crate ini;
extern crate tokio;
extern crate tokio_core;
extern crate tokio_io;
extern crate worklog;

use argparse::{ArgumentParser, Store};
use ini::Ini;
use std::io::{BufReader, Error, ErrorKind};
use std::iter;
use tokio::prelude::*;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::io;

fn main() {
    let mut config_file = "/etc/wlog.ini".to_string();

    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Work log server");
        ap.refer(&mut config_file).add_option(
            &["--config-file", "-c"],
            Store,
            "Path to the config file",
        );
        ap.parse_args_or_exit();
    }

    let conf = match Ini::load_from_file(&config_file) {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("Error loading config: {} at {}", e, config_file);

            return;
        }
    };

    let section = match conf.section(Some("Server")) {
        Some(section) => section,
        None => {
            eprintln!(
                "Could not read section 'Config' from configuration file in {}",
                config_file
            );

            return;
        }
    };

    let server = match section.get("listen") {
        Some(server) => server,
        None => {
            eprintln!("Missing key 'listen' in config section 'Server'");

            return;
        }
    };

    let port = match section.get("port") {
        Some(port) => port,
        None => {
            eprintln!("Missing key 'port' in config section 'Server'");

            return;
        }
    };

    let database_file = match section.get("database_file") {
        Some(database_file) => database_file.clone(),
        None => {
            eprintln!("Missing key 'database_file' in config section 'Server'");

            return;
        }
    };

    let addr = format!("{}:{}", server, port).parse().unwrap();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let listener = TcpListener::bind(&addr, &handle).expect("unable to bind TCP listener");

    let server = listener
        .incoming()
        .map_err(|e| eprintln!("accept failed = {:?}", e))
        .for_each(|(sock, _)| {
            let (reader, writer) = sock.split();
            let reader = BufReader::new(reader);
            let iter = stream::iter_ok::<_, Error>(iter::repeat(()));
            let database_file = database_file.clone();
            let (tx, rx) = futures::sync::mpsc::unbounded();
            let socket_reader = iter.fold(reader, move |reader, _| {
                // Read a line off the socket, failing if we're at EOF
                let line = io::read_until(reader, b'\n', Vec::new());

                let line = line.and_then(|(reader, vec)| {
                    if vec.len() == 0 {
                        Err(Error::new(ErrorKind::BrokenPipe, "broken pipe"))
                    } else {
                        Ok((reader, vec))
                    }
                });

                let line = line.map(|(reader, vec)| {
                    if let Ok(string) = String::from_utf8(vec) {
                        (reader, string)
                    } else {
                        (reader, String::from(""))
                    }
                });

                let database_file = database_file.clone();
                let this_tx = tx.clone();
                line.map(move |(reader, msg)| {
                    println!("{}: {:?}", addr, &msg);
                    this_tx
                        .unbounded_send(process_command(&msg, &database_file))
                        .unwrap();

                    reader
                })
            });

            let socket_writer = rx.fold(writer, |writer, msg: String| {
                let amt = io::write_all(writer, msg.into_bytes());
                let amt = amt.map(|(writer, _)| writer);
                amt.map_err(|_| ())
            });

            let socket_reader = socket_reader.map_err(|_| ());
            let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));

            handle.spawn(connection.then(move |_| {
                println!("Connection {} closed.", addr);
                Ok(())
            }));

            Ok(())
        });

    core.run(server).unwrap();
}

fn process_command(command: &str, database_file: &str) -> String {
    let command = command.trim();

    if command.eq_ignore_ascii_case("PING") {
        return "PONG\n".to_string();
    }

    let mut wlog = worklog::Wlog::new(database_file);

    if command.starts_with("LOG ") {
        let note = command.chars().skip(4).collect::<String>();

        wlog.log(&worklog::Entry::now(&note));

        return format!("LOGGED {}\n", &note);
    }

    if command.eq_ignore_ascii_case("DUMP") {
        let mut result = String::from("IMPORT ");

        result.push_str(
            &wlog
                .find_all()
                .into_iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join("\nIMPORT "),
        );

        result.push_str("\n\n");

        println!("{}", result);

        return result;
    }

    if command.starts_with("DUMP FROM ") {
        let date = command.chars().skip(10).collect::<String>();
        let mut result = String::from("");

        result.push_str(
            &wlog
                .find_by_date(&date)
                .into_iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(","),
        );

        return result;
    }

    if command.starts_with("IMPORT ") {
        wlog.sync(&worklog::Entry::from_json(&command.chars().skip(7).collect::<String>()));

        return String::from("OK\n");
    }

    format!("Error: Unknown command {}\n", command)
}
