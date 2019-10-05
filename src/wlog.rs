use chrono::prelude::*;
use rusqlite::{Connection, NO_PARAMS};

pub struct Entry {
    pub message: String,
    pub time_created: String
}

impl Entry {
    pub fn now(message: String) -> Entry {
        let dt = Local::now();
        let date = dt.format("%Y-%m-%d").to_string();
        
        Entry {
            message: message.trim().to_string(),
            time_created: date.to_string()
        }
    }

    pub fn from_past(date: String, message: String) -> Entry {
        Entry {
            message: message.trim().to_string(),
            time_created: date
        }
    }
}

pub struct Wlog {
    conn: Connection
}

impl Wlog {
    pub fn new(path: String) -> Wlog {
        let conn = Connection::open(&path).unwrap();

        conn.execute("CREATE TABLE IF NOT EXISTS entry (
                  message         TEXT NOT NULL,
                  time_created    TEXT NOT NULL
                  )", NO_PARAMS).unwrap();

        Wlog {conn}
    }

    pub fn log(&mut self, entry: Entry) {
        self.conn.execute("INSERT INTO entry (message, time_created) VALUES (?1, ?2)", &[&entry.message, &entry.time_created]).unwrap();
    }

    pub fn find_by_date(&mut self, date: &String) -> Vec<Entry> {
        let mut stmt = self.conn.prepare("SELECT message, time_created FROM entry WHERE time_created = ?").unwrap();

        stmt
        .query_map(&[&date], |row| Ok(Entry::from_past(row.get(1).unwrap(), row.get(0).unwrap())))
        .unwrap()
        .map(|e| e.unwrap())
        .collect::<Vec<Entry>>()   
    }    

    pub fn find_by_message(&mut self, message: &String) -> Vec<Entry> {
        let mut stmt = self.conn.prepare("SELECT message, time_created FROM entry WHERE message like ?").unwrap();

        stmt
        .query_map(&[&message], |row| Ok(Entry::from_past(row.get(1).unwrap(), row.get(0).unwrap())))
        .unwrap()
        .map(|e| e.unwrap())
        .collect::<Vec<Entry>>()         
    }    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entry() {
        let e = Entry::now(String::from("new entry"));

        let dt = Local::now();
        let date = dt.format("%Y-%m-%d").to_string();
        assert_eq!("new entry", e.message);
        assert_eq!(date, e.time_created);
    }

    #[test]
    fn test_new_entry_from_past() {
        let e = Entry::from_past(String::from("2019-01-01"), String::from("new entry"));

        assert_eq!("new entry", e.message);
        assert_eq!("2019-01-01", e.time_created);
    }
}
