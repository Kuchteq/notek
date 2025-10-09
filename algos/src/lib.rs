use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{char, cmp::Ordering, collections::BTreeMap};

mod msg;
pub use msg::PeerMessage;

const LBASE: u32 = u32::MAX; // maximum identifier value
/// A single position in a PID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pos {
    ident: u32,
    site: u8,
}

impl Pos {
    pub fn new(ident: u32, site: u8) -> Pos {
        Pos { ident, site }
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

/// A PID is a vector of positions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pid(Vec<Pos>);

impl Pid {
    pub fn new(ident: u32) -> Pid {
        Pid(vec![Pos::new(ident, 1)])
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Doc {
    content: BTreeMap<Pid, char>,
    pub site: u8,
}

impl Doc {
    pub fn new(content: String) -> Doc {
        let beg = (Pid(vec![Pos { ident: 0, site: 0 }]), '_');
        let end = (
            Pid(vec![Pos {
                ident: LBASE,
                site: 0,
            }]),
            '_',
        );

        let mut d = Doc {
            content: BTreeMap::from([beg.clone(), end.clone()]),
            site: 1,
        };
        let step = (u32::MAX as usize) / content.len();
        for (i, c) in content.chars().enumerate() {
            d.content.insert(
                Pid(vec![Pos {
                    ident: (i * step) as u32,
                    site: 1,
                }]),
                c,
            );
        }
        d
    }
    pub fn right(&self, pid: &Pid) -> Pid {
        self.content.range(pid..).skip(1).next().unwrap().0.clone()
    }
    pub fn left(&self, pid: &Pid) -> Pid {
        self.content.range(..pid).next_back().unwrap().0.clone()
    }

    pub fn offset(&self, pid: &Pid, offset: isize) -> Option<Pid> {
        if offset == 0 {
            return Some(pid.clone());
        }

        if offset > 0 {
            // move right
            self.content
                .range(pid..) // start at pid
                .skip(1) // skip self
                .nth((offset - 1) as usize)
                .map(|(k, _)| k.clone())
        } else {
            // move left
            self.content
                .range(..pid) // all before pid
                .rev() // backwards
                .nth((-offset - 1) as usize)
                .map(|(k, _)| k.clone())
        }
    }
    pub fn keys(&self) -> Vec<Pid> {
        self.content.keys().cloned().collect()
    }
    pub fn values(&self) -> Vec<char> {
        self.content.values().cloned().collect()
    }
    // fn index(&self, pid: &Pid) -> usize {
    //     for (i, p) in self.pairs.iter().enumerate() {
    //         if compare_pid(&p.pid, &pid) == 0 {
    //             return i;
    //         }
    //     }
    //     return 0;
    // }
    //

    // pub fn index(&self, pid: &Pid) -> (usize, bool) {
    //     let mut search_scope: &[Pair] = &self.content;
    //     let mut off = 0;
    //
    //     loop {
    //         if search_scope.is_empty() {
    //             return (off, false);
    //         }
    //
    //         let split_point = search_scope.len() / 2;
    //         let mid = &search_scope[split_point].pid;
    //
    //         match pid.cmp(mid) {
    //             Ordering::Equal => return (off + split_point, true),
    //             Ordering::Less => {
    //                 search_scope = &search_scope[..split_point]; // left half
    //             }
    //             Ordering::Greater => {
    //                 off += split_point + 1;
    //                 search_scope = &search_scope[split_point + 1..];
    //             }
    //         }
    //     }
    // }

    pub fn insert(&mut self, pid: Pid, c: char) {
        self.content.insert(pid, c);
    }
    pub fn insert_left(&mut self, pid: Pid, c: char) -> Pid {
        let right = self.right(&pid);
        let new = generate_between_pids(&pid, &right, 1);
        self.content.insert(new.clone(), c);
        return new;
    }

    pub fn delete(&mut self, pid: &Pid) {
        self.content.remove(pid);
    }
    // pub fn insert_idx(&mut self, idx: usize, c: char) {
    //     let l = &self.content[idx].pid;
    //     let r = &self.content[idx + 1].pid;
    //     let p = generate_between_pids(l, r, 1);
    //     let p = Pair { pid: p, atom: c };
    //     self.content.insert(idx + 1, p)
    // }
    pub fn to_string(&self) -> String {
        self.content.values().collect()
    }
    pub fn len(&self) -> usize {
        return self.content.len();
    }
}
