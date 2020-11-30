extern crate chrono;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use chrono::prelude::*;
use rusqlite::{Connection, NO_PARAMS};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::result::Result;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub message: String,
    pub time_created: String,
}

impl Entry {
    pub fn now(message: &String) -> Self {
        let dt = Local::now();
        let date = dt.format("%Y-%m-%d").to_string();

        Self {
            id: Uuid::new_v4(),
            message: message.trim().to_string(),
            time_created: date,
        }
    }

    pub fn from_date(id: &Option<String>, date: &String, message: &String) -> Self {
        if let Some(id) = id {
            return Self {
                id: Uuid::parse_str(id).unwrap(),
                message: message.trim().to_string(),
                time_created: date.to_string(),
            };
        }

        Self {
            id: Uuid::new_v4(),
            message: message.trim().to_string(),
            time_created: date.to_string(),
        }
    }

    pub fn from_json(serialized: &str) -> Self {
        serde_json::from_str(serialized).unwrap()
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&serde_json::to_string(self).unwrap())
    }
}

pub struct Wlog {
    conn: Connection,
}

impl Wlog {
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(&path).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS entry (
                  id              TEXT NOT NULL,
                  message         TEXT NOT NULL,
                  time_created    TEXT NOT NULL
                  )",
            NO_PARAMS,
        )
        .unwrap();

        Self { conn }
    }

    pub fn log(&mut self, entry: &Entry) {
        self.conn
            .execute(
                "INSERT INTO entry (id, message, time_created) VALUES (?1, ?2, ?3)",
                &[&entry.id.to_string(), &entry.message, &entry.time_created],
            )
            .unwrap();
    }

    pub fn sync(&mut self, entry: &Entry) -> bool {
        let id = &entry.id.to_string();

        if self.find_by_id(&id).len() == 0 {
            self.conn
                .execute(
                    "INSERT INTO entry (id, message, time_created) VALUES (?1, ?2, ?3)",
                    &[&id, &entry.message, &entry.time_created],
                )
                .unwrap();
            true
        } else {
            false
        }
    }

    pub fn find_by_id(&mut self, id: &str) -> Vec<Entry> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, time_created, message FROM entry WHERE id = ?1")
            .unwrap();

        stmt.query_map(&[&id], |row| {
            Ok(Entry::from_date(
                &Some(row.get(0).unwrap()),
                &row.get(1).unwrap(),
                &row.get(2).unwrap(),
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<Entry>>()
    }

    pub fn find_all(&self) -> Vec<Entry> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, time_created, message FROM entry")
            .unwrap();

        stmt.query_map(NO_PARAMS, |row| {
            Ok(Entry::from_date(
                &Some(row.get(0).unwrap()),
                &row.get(1).unwrap(),
                &row.get(2).unwrap(),
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<Entry>>()
    }

    pub fn find_by_date(&mut self, date: &str) -> Vec<Entry> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, time_created, message  FROM entry WHERE time_created = ?")
            .unwrap();

        stmt.query_map(&[&date], |row| {
            Ok(Entry::from_date(
                &Some(row.get(0).unwrap()),
                &row.get(1).unwrap(),
                &row.get(2).unwrap(),
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<Entry>>()
    }

    pub fn find_by_message(&mut self, message: &str) -> Vec<Entry> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, time_created, message FROM entry WHERE message like ?")
            .unwrap();

        stmt.query_map(&[&message], |row| {
            Ok(Entry::from_date(
                &Some(row.get(0).unwrap()),
                &row.get(1).unwrap(),
                &row.get(2).unwrap(),
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<Entry>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entry() {
        let e = Entry::now(&String::from("new entry"));

        let dt = Local::now();
        let date = dt.format("%Y-%m-%d").to_string();
        assert_eq!("new entry", e.message);
        assert_eq!(date, e.time_created);
    }

    #[test]
    fn test_new_entry_from_date_with_id() {
        let e = Entry::from_date(
            &Some(String::from("c1a69488-d452-4f23-9567-b81138b04096")),
            &String::from("2019-01-01"),
            &String::from("new entry"),
        );

        assert_eq!("c1a69488-d452-4f23-9567-b81138b04096", e.id.to_string());
        assert_eq!("new entry", e.message);
        assert_eq!("2019-01-01", e.time_created);
    }

    #[test]
    fn test_new_entry_from_date_without_id() {
        let e = Entry::from_date(
            &None,
            &String::from("2019-01-01"),
            &String::from("new entry"),
        );

        assert_eq!("new entry", e.message);
        assert_eq!("2019-01-01", e.time_created);
    }
}
