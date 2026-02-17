use std::{collections::BTreeSet, time::Instant};

use rand::{rng, seq::SliceRandom};

struct Node {
    keys: Vec<usize>,
    children: Vec<Node>,
    size: usize,
    is_leaf: bool,
}

struct MarTree {
    root: Node,
}

impl Default for MarTree {
    fn default() -> Self {
        MarTree {
            root: Node {
                keys: Vec::new(),
                children: Vec::new(),
                size: 0,
                is_leaf: true,
            },
        }
    }
}

const T: usize = 4;

impl Default for Node {
    fn default() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
            size: 0,
            is_leaf: true,
        }
    }
}

impl Node {
    // The function name is a precondition:
    // Insert key into the subtree rooted at node, assuming node itself is not full
    fn insert_non_full(&mut self, val: usize) {
        let mut i = self.keys.len();
        if self.is_leaf {
            self.keys.push(0);
            while i > 0 && val < self.keys[i - 1] {
                self.keys[i] = self.keys[i - 1];
                i -= 1;
            }
            self.keys[i] = val
        } else {
            while i > 0 && val < self.keys[i - 1] {
                i -= 1;
            }
            if self.children[i].keys.len() >= T * 2 - 1 {
                self.split_child(i);

                if val > self.keys[i] {
                    i += 1;
                }
            }
            self.children[i].insert_non_full(val);
            self.children[i].size = self.children[i].recompute_size();
        }
    }
    fn recompute_size(&self) -> usize {
        if self.is_leaf {
            self.keys.len()
        } else {
            self.keys.len() + self.children.iter().map(|c| c.size).sum::<usize>()
        }
    }

    fn split_child(&mut self, i: usize) {
        // y is the full child to be split
        let is_leaf = self.children[i].is_leaf;

        // Split keys
        let right_keys = self.children[i].keys.split_off(T);
        let mid = self.children[i].keys.pop().unwrap(); // median key

        // Split children if internal node
        let right_children;
        if is_leaf {
            right_children = Vec::new();
        } else {
            right_children = self.children[i].children.split_off(T);
        };

        let right = Node {
            keys: right_keys,
            children: right_children,
            size: 0,
            is_leaf,
        };
        // Insert median key and new child into parent
        self.keys.insert(i, mid);
        self.children.insert(i + 1, right);
        self.children[i].size = self.children[i].recompute_size();
    }
    fn total_keys(&self) -> usize {
        let mut sum = self.keys.len();
        for child in &self.children {
            sum += child.total_keys();
        }
        sum
    }
    fn remove(&mut self, val: usize) {
        let mut idx = 0;

        while idx < self.keys.len() && self.keys[idx] < val {
            idx += 1
        }
        if idx < self.keys.len() && self.keys[idx] == val {
            if self.is_leaf {
                self.keys.remove(idx);
                return;
            }
            // If not a leaf then its an internal node
            if self.children[idx].keys.len() >= T {
                let predecessor = self.get_predecessor(idx);
                self.keys[idx] = predecessor;
                self.children[idx].remove(predecessor);
            } else if self.children[idx + 1].keys.len() >= T {
                let successor = self.get_successor(idx);
                self.keys[idx] = successor;
                self.children[idx + 1].remove(successor);
            } else {
                self.merge_children(idx);
                self.children[idx].remove(val);
            }
            return;
        }
        // otherwise we need to descend deeper

        // if the child needs a fixup
        if self.children[idx].keys.len() == T - 1 {
            if idx >= 1 && self.children[idx - 1].keys.len() >= T {
                self.borrow_left(idx);
                self.children[idx].remove(val);
            } else if idx + 1 < self.children.len() && self.children[idx + 1].keys.len() >= T {
                self.borrow_right(idx);
                self.children[idx].remove(val);
            // Investigate this if it really works
            } else if idx == self.keys.len() {
                self.merge_children(idx - 1);
                self.children[idx - 1].remove(val);
            } else {
                self.merge_children(idx);
                self.children[idx].remove(val);
            }
        } else {
            self.children[idx].remove(val);
        }
    }
    fn borrow_left(&mut self, idx: usize) {
        let parent = self.keys[idx - 1];
        self.children[idx].keys.insert(0, parent);
        let left_key = self.children[idx - 1].keys.pop().unwrap();
        self.keys[idx - 1] = left_key;
        if !self.children[idx - 1].is_leaf {
            let left_last_child = self.children[idx - 1].children.pop().unwrap();
            self.children[idx].children.insert(0, left_last_child);
        }
    }
    fn borrow_right(&mut self, idx: usize) {
        let parent = self.keys[idx];
        self.children[idx].keys.push(parent);
        let right_key = self.children[idx + 1].keys.remove(0);
        self.keys[idx] = right_key;
        if !self.children[idx + 1].is_leaf {
            let right_first_child = self.children[idx + 1].children.remove(0);
            self.children[idx].children.push(right_first_child);
        }
    }

    fn merge_children(&mut self, idx: usize) {
        let right = self.children.remove(idx + 1);
        let child = &mut self.children[idx];

        child.keys.push(self.keys.remove(idx));
        child.keys.extend(right.keys);

        if !child.is_leaf {
            child.children.extend(right.children);
        }
    }

    fn get_predecessor(&self, idx: usize) -> usize {
        let mut n = &self.children[idx];
        while !n.is_leaf {
            n = &n.children.last().unwrap()
        }
        return *n.keys.last().unwrap();
    }
    fn get_successor(&self, idx: usize) -> usize {
        let mut n = &self.children[idx + 1];
        while !n.is_leaf {
            n = &n.children.first().unwrap()
        }
        return *n.keys.first().unwrap();
    }
    fn print(&self, depth: usize) {
        let indent = "  ".repeat(depth);

        print!("{}", indent);
        print!("[");
        for (i, k) in self.keys.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", k);
        }
        println!("]");

        if !self.is_leaf {
            for (i, child) in self.children.iter().enumerate() {
                print!("{}  ({}) 'c:{}'", indent, i, self.children[i].size);
                child.print(depth + 1);
            }
        }
    }

    fn validate(&self, is_root: bool) {
        // Check key bounds
        if !is_root {
            assert!(
                self.keys.len() >= T - 1,
                "Node underflow: {} keys",
                self.keys.len()
            );
        }

        assert!(
            self.keys.len() <= 2 * T - 1,
            "Node overflow: {} keys",
            self.keys.len()
        );

        // Check sorted keys
        for i in 1..self.keys.len() {
            assert!(self.keys[i - 1] < self.keys[i]);
        }

        if !self.is_leaf {
            assert_eq!(
                self.children.len(),
                self.keys.len() + 1,
                "Children mismatch"
            );

            for child in &self.children {
                child.validate(false);
            }
        }
    }
}

impl MarTree {
    fn insert(&mut self, val: usize) {
        if self.root.keys.len() >= T * 2 - 1 {
            let old_root = std::mem::take(&mut self.root);
            let s = old_root.recompute_size();
            let mut new_root = Node {
                keys: Vec::new(),
                children: vec![old_root],
                size: s,
                is_leaf: false,
            };
            new_root.split_child(0);
            new_root.insert_non_full(val);
            self.root = new_root;
        } else {
            self.root.insert_non_full(val);
        }
    }
    fn remove(&mut self, val: usize) {
        self.root.remove(val);
        if self.root.keys.is_empty() && !self.root.is_leaf {
            self.root = self.root.children.remove(0);
        }
    }
    fn validate(&self) {
        self.root.validate(true);
    }
    fn print(&self, depth: usize) {
        self.root.print(depth);
    }
}

fn main() {
    const N: usize = 100_000; // number of elements to insert

    // Generate a sequence of numbers (you can shuffle for random order)
    let mut values: Vec<usize> = (0..N).collect();

    let mut rng = rng();

    values.shuffle(&mut rng);
    // --- Test your custom BTree ---
    let mut my_tree = MarTree {
        root: Node {
            keys: Vec::new(),
            children: Vec::new(),
            size: 0,
            is_leaf: true,
        },
    };

    let start = Instant::now();
    for &v in &values {
        my_tree.insert(v);
    }
    let duration = start.elapsed();
    println!(
        "Custom B-tree insertion of {} elements took: {:?}",
        N, duration
    );

    // Optional: sanity check (e.g., print root keys)
    println!("Custom B-tree root keys count: {}", my_tree.root.keys.len());

    // --- Test Rust's standard BTreeMap ---
    let mut std_tree = BTreeSet::new();

    let start = Instant::now();
    for &v in &values {
        std_tree.insert(v);
    }
    let duration = start.elapsed();
    println!(
        "std::BTreeMap insertion of {} elements took: {:?}",
        N, duration
    );

    println!("std::BTreeMap size: {}", std_tree.len());
    println!("Custom B-tree total keys: {}", my_tree.root.total_keys());

    let mut visualizer = MarTree::default();
    visualizer.insert(1);
    visualizer.insert(2);
    visualizer.insert(3);
    visualizer.insert(4);
    visualizer.insert(1);
    visualizer.insert(8);
    visualizer.insert(9);
    visualizer.insert(18);
    visualizer.insert(0);
    visualizer.insert(8);
    visualizer.insert(4);
    visualizer.print(2);
    // --- Benchmark deletion for custom B-tree ---

    // let mut values_to_delete = values.clone();
    // values_to_delete.shuffle(&mut rng);
    //
    // let start = Instant::now();
    // for &v in &values_to_delete {
    //     my_tree.remove(v);
    // }
    // let duration = start.elapsed();
    //
    // println!(
    //     "Custom B-tree deletion of {} elements took: {:?}",
    //     N, duration
    // );
    //
    // println!(
    //     "Custom B-tree total keys after deletion: {}",
    //     my_tree.root.total_keys()
    // );
    //
    // // --- Benchmark deletion for std::BTreeSet ---
    //
    // let mut std_values = values.clone();
    // std_values.shuffle(&mut rng);
    //
    // let start = Instant::now();
    // for &v in &std_values {
    //     std_tree.remove(&v);
    // }
    // let duration = start.elapsed();
    //
    // println!(
    //     "std::BTreeSet deletion of {} elements took: {:?}",
    //     N, duration
    // );
    //
    // println!("std::BTreeSet size after deletion: {}", std_tree.len());
    //
}
