use std::{fs::File};
use rusqlite::{ Connection };
pub struct Persistence {
    pub connection: Option<Connection>,
}

impl Persistence {

    pub fn new() -> Self {
        Self::create_database();

        Persistence { connection: Some(Connection::open("tasks.db").unwrap()) }
    }


    pub fn sync_schema(&self) {
        if let Some(conn) = &self.connection {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id BIGINT PRIMARY KEY,
                    title TEXT NOT NULL,
                    description TEXT,
                    completed BOOLEAN NOT NULL
                )",
                [],
            ).expect("Failed to create tasks table");
        }
    }

    fn create_database() {
        let file: Result<File, std::io::Error> = File::open("tasks.db");
        match file {
            Ok(_) => (),
            Err(_) => {
                match File::create("tasks.db") {
                    Ok(_) => (),
                    Err(e) => eprintln!("Failed to create database: {}", e),
                }
            }
        }
    }

    pub fn save<T: Persistable>(&self, item: &T) {
        if let Some(conn) = &self.connection {
            conn.execute(
                item.insert_sql().as_str(),
                item.params().as_slice()
            ).expect("Failed to insert item");
        }   
    }

    pub fn get_all<T: Persistable>(&self) -> Vec<T> {
        let mut items = Vec::new();
        if let Some(conn) = &self.connection {
            let mut stmt = conn.prepare(T::get_all_sql().as_str()).expect("Failed to prepare statement");
            let rows = stmt.query_map([], |row| T::from_row(row)).expect("Failed to query items");

            for item in rows {
                items.push(item.expect("Failed to map item"));
            }
        }
        items
    }

    pub fn delete<T: Persistable>(&self, id: i64) {
        if let Some(conn) = &self.connection {
            conn.execute(T::delete_sql().as_str(), [id])
                .expect("Failed to delete item");
        }
    }

}



pub trait Persistable: Sized {
    fn insert_sql(&self) -> String;
    fn params(&self) -> Vec<&dyn rusqlite::ToSql>;
    fn get_all_sql() -> String;
    fn delete_sql() -> String;
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self>;
    fn describe(&self) -> String;
}

pub struct Task {
    pub id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
}

impl Persistable for Task {
    fn insert_sql(&self) -> String {
        "INSERT INTO tasks (title, description, completed) VALUES (?1, ?2, ?3)".to_string()
    }

    fn params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.title, &self.description, &self.completed]
    }

    fn get_all_sql() -> String {
        "SELECT id, title, description, completed FROM tasks".to_string()
    }

    fn delete_sql() -> String {
        "DELETE FROM tasks WHERE id = ?1".to_string()
    }

    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            completed: row.get(3)?,
        })
    }

    fn describe(&self) -> String {
        format!("ID: {}, Title: {}, Description: {:?}, Completed: {}",
            self.id.unwrap_or(0),
            self.title,
            self.description.as_deref().unwrap_or("no description"),
            self.completed 
        )
    }
}