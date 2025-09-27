use std::fs;

use rand::Rng;

type Pid = Vec<Pos>;

const LBASE: u32 = u32::MAX; // maximum identifier value

fn generate_between_pids(lp: &Pid, rp: &Pid) -> Pid {
    let mut p = Vec::new();

    let max_depth = lp.len().max(rp.len());

    for i in 0..max_depth {
        let l = lp.get(i).cloned().unwrap_or(Pos { ident: 0, site: 0 });
        let r = rp.get(i).cloned().unwrap_or(Pos {
            ident: LBASE,
            site: 0,
        });

        if l.ident == r.ident {
            // Keep the common prefix
            p.push(l.clone());
            continue;
        }

        let d = r.ident.saturating_sub(l.ident);

        if d > 1 {
            // Found space → pick midpoint
            let mut rng = rand::rng();
            let new_ident = rng.random_range(l.ident + 1..r.ident);
            // let new_ident = l.ident + d / 2;
            p.push(Pos {
                ident: new_ident,
                site: 1, // TODO: assign site ID properly
            });
            return p;
        } else {
            // No space, must go deeper
            p.push(l.clone());
        }
    }

    // If we reached here, append a new level at the end
    p.push(Pos {
        ident: LBASE / 2,
        site: 1,
    });

    p
}

// ComparePos compares two pid, returning -1 if the left is less than the
// right, 0 if equal, and 1 if greater.
fn compare_pid(lp: &Pid, rp: &Pid) -> i8 {
    if rp.len() < lp.len() {
        return 1;
    } else if rp.len() > lp.len() {
        return -1;
    }
    for i in 0..lp.len() {
        let l = &lp[i];
        let r = &rp[i];

        if r.ident < l.ident {
            return 1;
        }
        if r.ident > l.ident {
            return -1;
        }
    }
    return 0;
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pos {
    ident: u32,
    site: u8,
}

impl PartialEq for Pos {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident && self.site == other.site
    }
}

#[derive(Debug)]
struct Pair {
    pid: Pid,
    atom: char,
}

#[derive(Debug)]
struct Doc {
    pairs: Vec<Pair>,
    site: u8,
}

impl Doc {
    fn new(content: String) -> Doc {
        let beg = Pair {
            pid: vec![Pos { ident: 0, site: 0 }],
            atom: '_',
        };
        let end = Pair {
            pid: vec![Pos {
                ident: LBASE,
                site: 0,
            }],
            atom: '_',
        };

        let mut d = Doc {
            pairs: vec![beg, end],
            site: 1,
        };

        for c in content.chars() {
            d.insert_idx(d.len() - 2, c);
        }
        d
    }
    fn index(&self, pid: &Pid) -> usize {
        for (i, p) in self.pairs.iter().enumerate() {
            if compare_pid(&p.pid, &pid) == 0 {
                return i;
            }
        }
        return 0;
    }

    fn insert(&mut self, pid: &Pid, c: char) {}

    fn insert_idx(&mut self, idx: usize, c: char) {
        let l = &self.pairs[idx].pid;
        let r = &self.pairs[idx + 1].pid;
        let p = generate_between_pids(l, r);
        let p = Pair { pid: p, atom: c };
        self.pairs.insert(idx + 1, p)
    }

    fn len(&self) -> usize {
        return self.pairs.len();
    }
}

fn main() {
    let content = fs::read_to_string("foo.txt").unwrap();
    let doc = Doc::new(content.to_string());
    println!("{:#?}", doc.len());
}
