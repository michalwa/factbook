use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use chrono::NaiveDateTime;

use crate::model::Entry;

pub fn read_journal_file(filename: &str) -> Result<Vec<Entry>, JournalError> {
    BufReader::new(File::open(filename)?)
        .lines()
        .map(|line| {
            let line = line?;
            match line.split_once("|") {
                Some((timestamp, content)) => Ok(Entry {
                    timestamp: NaiveDateTime::parse_from_str(timestamp.trim(), "%Y-%m-%d %H:%M")?,
                    content: content.trim().to_owned(),
                }),
                None => Err(JournalError::MalformedEntry(line)),
            }
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Malformed timestamp: {0}")]
    Timestamp(#[from] chrono::ParseError),
    #[error("Malformed entry: {0}")]
    MalformedEntry(String),
}
