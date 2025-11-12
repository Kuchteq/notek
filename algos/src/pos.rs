use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering, io::Write,
};
use anyhow::{anyhow, Context, Result};

/// A single position in a PID
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Pos {
    pub ident: u32,
    pub site: u8,
}
impl Pos {
    pub fn new(ident: u32, site: u8) -> Pos {
        Pos { ident, site }
    }
    pub fn write_bytes_tobuf(&self, buf: &mut Vec<u8>) {
        buf.extend(&self.ident.to_le_bytes());
        buf.push(self.site);
    }
    pub fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.ident.to_le_bytes())?;
        writer.write(&[self.site])?;
        Ok(())
    }
}

impl PartialEq for Pos {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident && self.site == other.site
    }
}
impl Eq for Pos {}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Pos {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.ident.cmp(&other.ident) {
            Ordering::Equal => self.site.cmp(&other.site),
            ord => ord,
        }
    }
}

