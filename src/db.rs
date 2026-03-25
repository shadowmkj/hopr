use std::{
    fmt::Display,
    fs::{self, canonicalize},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};

use crate::utils::{self, DAY, HOUR, WEEK};

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Record {
    pub path: PathBuf,
    pub score: Rank,
    pub last_accessed: Epoch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub path: PathBuf,
    pub records: Vec<Record>,
}

impl Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in &self.records {
            let _ = writeln!(f, "{:?}: {}", r.path, r.score);
        }
        Ok(())
    }
}

impl Record {
    pub fn new(path: PathBuf) -> Self {
        Record {
            path,
            score: 1.0,
            last_accessed: 0,
        }
    }

    pub fn score(&self, now: Epoch) -> Rank {
        // The older the entry, the lesser its importance.
        let duration = now.saturating_sub(self.last_accessed);
        if duration < HOUR {
            self.score * 4.0
        } else if duration < DAY {
            self.score * 2.0
        } else if duration < WEEK {
            self.score * 0.5
        } else {
            self.score * 0.25
        }
    }
}

impl Database {
    pub fn new(path: PathBuf) -> Self {
        Database {
            path,
            records: vec![],
        }
    }

    pub fn load(&mut self) -> Result<Self> {
        let file_bytes = fs::read(&self.path)?;
        let decoded_db: Database = from_bytes(&file_bytes)?;
        Ok(decoded_db)
    }

    pub fn add(&mut self, file: PathBuf) {
        let file = Record::new(file);
        self.records.push(file);
    }

    pub fn save(&mut self) -> Result<()> {
        let encoded_bytes = to_allocvec(self)?;
        fs::write(&self.path, encoded_bytes)?;
        Ok(())
    }

    pub fn query(&mut self, file: &str) -> Result<&Record> {
        let now = utils::current_time()?;
        let best_idx = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, record)| record.path.to_string_lossy().contains(file))
            .max_by(|(_, a), (_, b)| a.score(now).total_cmp(&b.score(now)))
            .map(|(idx, _)| idx);

        if let Some(idx) = best_idx {
            self.records[idx].score = self.records[idx].score * 2.0;
            self.records[idx].last_accessed = now;
            self.save()?;
            Ok(&self.records[idx])
        } else {
            let rec = Record::new(canonicalize(file)?);
            self.records.push(rec);
            self.records.iter_mut().last().unwrap().last_accessed = now;
            self.save()?;

            Err(anyhow!("Did not find record"))
        }
    }
}

pub type Epoch = u64;
pub type Rank = f64;
