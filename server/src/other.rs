
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
