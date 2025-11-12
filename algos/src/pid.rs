use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    io::{Read, Write},
};
use anyhow::{anyhow, Context, Result};

use crate::{pos::Pos, LBASE};

/// A PID is a vector of positions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pid(pub Vec<Pos>);

impl Pid {
    pub fn new(ident: u32) -> Pid {
        Pid(vec![Pos::new(ident, 1)])
    }
    pub fn write_bytes_buf(&self, buf: &mut Vec<u8>) {
        for pos in &self.0 {
            pos.write_bytes_tobuf(buf);
        }
    } 
    pub fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<()> {
        for pos in &self.0 {
            pos.write_bytes(writer)
                .context("Failed to write Pid position")?;
        }
        Ok(())
    }
    pub fn depth(&self) -> usize {
        self.0.len()
    }

    pub fn from_reader<R: Read>(reader: &mut R, depth: usize) -> Self {
        let mut positions = Vec::with_capacity(depth);

        for _ in 0..depth {
            let mut ident_bytes = [0u8; 4];
            reader.read_exact(&mut ident_bytes).unwrap();
            let ident = u32::from_le_bytes(ident_bytes);

            let mut site = [0u8; 1];
            reader.read_exact(&mut site).unwrap();

            positions.push(Pos::new(ident, site[0]));
        }
        Pid(positions)
    }
}

impl PartialOrd for Pid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Pid {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lexicographic comparison of positions
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            match a.cmp(b) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            }
        }
        // If all common prefix equal → shorter wins
        self.0.len().cmp(&other.0.len())
    }
}

/// Generate a PID between two existing PIDs
pub fn generate_between_pids(lp: &Pid, rp: &Pid, site_id: u8) -> Pid {
    let mut p = Vec::new();

    let max_depth = lp.0.len().max(rp.0.len());
    let mut rng = rand::rng();

    for i in 0..max_depth {
        let l = lp.0.get(i).cloned().unwrap_or(Pos { ident: 0, site: 0 });
        let r = rp.0.get(i).cloned().unwrap_or(Pos {
            ident: LBASE,
            site: u8::MAX,
        });

        if l == r {
            p.push(l);
            continue;
        }

        if l.ident == r.ident {
            // same ident, different site → site_id tie-breaker
            p.push(Pos {
                ident: l.ident,
                site: site_id,
            });
            return Pid(p);
        }

        let d = r.ident.saturating_sub(l.ident);
        if d > 1 {
            let new_ident = rng.random_range(l.ident + 1..r.ident);
            p.push(Pos {
                ident: new_ident,
                site: site_id,
            });
            return Pid(p);
        } else {
            p.push(l);
        }
    }

    // If no gap found, extend depth
    p.push(Pos {
        ident: rng.random_range(0..LBASE),
        site: site_id,
    });

    Pid(p)
}

