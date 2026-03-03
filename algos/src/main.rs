use std::{collections::BTreeSet, time::Instant};

use indextreemap::{IndexTreeMap, IndexTreeSet};
use rand::{rng, seq::SliceRandom};

struct Node<K, V>
where
    K: Ord,
{
    keys: Vec<(K, V)>,
    children: Vec<Node<K, V>>,
    size: usize,
    is_leaf: bool,
}

struct MarTree<K, V>
where
    K: Ord,
{
    root: Node<K, V>,
}

impl<K: Ord, V> Default for MarTree<K, V> {
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

const T: usize = 6;

impl<K: Ord, V> Default for Node<K, V> {
    fn default() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
            size: 0,
            is_leaf: true,
        }
    }
}

impl<K: Ord, V> Node<K, V> {
    // The function name is a precondition:
    // Insert key into the subtree rooted at node, assuming node itself is not full
    fn insert_non_full(&mut self, key: K, value: V) {
        // Read the binary_search_by documentation that states what the Err means
        match self.keys.binary_search_by(|(k, _)| k.cmp(&key)) {
            Ok(pos) => {
                self.keys[pos].1 = value;
            }

            // ❌ Key not in this node
            Err(mut idx) => {
                // TODO to się spierdoli jak ktoś będzie chciał dodać istniejące gówno
                self.size += 1;
                if self.is_leaf {
                    self.keys.insert(idx, (key, value));
                } else {
                    if self.children[idx].keys.len() == 2 * T - 1 {
                        self.split_child(idx);

                        // After split, decide correct child
                        if key > self.keys[idx].0 {
                            idx += 1;
                        }
                    }

                    self.children[idx].insert_non_full(key, value);
                }
            }
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
        self.children[i + 1].size = self.children[i + 1].recompute_size();
    }
    fn total_keys(&self) -> usize {
        let mut sum = self.keys.len();
        for child in &self.children {
            sum += child.total_keys();
        }
        sum
    }
    fn remove(&mut self, key: &K) -> bool {
        let mut idx = 0;
        while idx < self.keys.len() && &self.keys[idx].0 < key {
            idx += 1;
        }

        // -------------------------
        // CASE 1: Key found here
        // -------------------------
        if idx < self.keys.len() && &self.keys[idx].0 == key {
            // 1A: Leaf
            if self.is_leaf {
                self.keys.remove(idx);
                self.size -= 1;
                return true;
            }

            // 1B: Internal node
            if self.children[idx].keys.len() >= T {
                let pred = self.pop_predecessor(idx);
                self.keys[idx] = pred;
                self.size -= 1;
                return true;
            } else if self.children[idx + 1].keys.len() >= T {
                let succ = self.pop_successor(idx);
                self.keys[idx] = succ;
                self.size -= 1;
                return true;
            } else {
                self.merge_children(idx);
                let deleted = self.children[idx].remove(key);
                if deleted {
                    self.size -= 1;
                }
                return deleted;
            }
        }

        // -------------------------
        // CASE 2: Not found here
        // -------------------------

        if self.is_leaf {
            return false;
        }

        // Ensure child has ≥ T keys before descending
        if self.children[idx].keys.len() == T - 1 {
            if idx > 0 && self.children[idx - 1].keys.len() >= T {
                self.borrow_left(idx);
            } else if idx + 1 < self.children.len() && self.children[idx + 1].keys.len() >= T {
                self.borrow_right(idx);
            } else {
                if idx < self.keys.len() {
                    self.merge_children(idx);
                } else {
                    self.merge_children(idx - 1);
                    idx -= 1;
                }
            }
        }

        let deleted = self.children[idx].remove(key);
        if deleted {
            self.size -= 1;
        }
        deleted
    }
    fn borrow_left(&mut self, idx: usize) {
        // Move parent key into child
        let (left_slice, right_slice) = self.children.split_at_mut(idx);

        let left = &mut left_slice[idx - 1];
        let right = &mut right_slice[0];

        // 1 Take last key from left sibling
        let borrowed_key = left.keys.pop().unwrap();

        // 2 Swap with parent key
        let parent_key = std::mem::replace(&mut self.keys[idx - 1], borrowed_key);

        // 3 Insert old parent key into right child
        right.keys.insert(0, parent_key);

        // Size adjustments for key movement
        left.size -= 1;
        right.size += 1;

        if !left.is_leaf {
            let moved_child = left.children.pop().unwrap();
            let moved_size = moved_child.size;

            right.children.insert(0, moved_child);

            // Adjust sizes for subtree movement
            left.size -= moved_size;
            right.size += moved_size;
        }
    }
    // borrows *from* right child to left one
    fn borrow_right(&mut self, idx: usize) {
        let (left_slice, right_slice) = self.children.split_at_mut(idx + 1);

        let left = &mut left_slice[idx];
        let right = &mut right_slice[0];

        let borrowed_key = right.keys.remove(0);
        let parent_key = std::mem::replace(&mut self.keys[idx], borrowed_key);
        left.keys.push(parent_key);

        left.size += 1;
        right.size -= 1;

        if !right.is_leaf {
            let moved_child = right.children.remove(0);
            let moved_size = moved_child.size;

            left.children.push(moved_child);

            left.size += moved_size;
            right.size -= moved_size;
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

        child.size += right.size + 1;
    }

    fn get_predecessor(&self, idx: usize) -> &(K, V) {
        let mut n = &self.children[idx];
        while !n.is_leaf {
            n = &n.children.last().unwrap();
        }
        return n.keys.last().unwrap();
    }
    fn pop_predecessor(&mut self, idx: usize) -> (K, V) {
        let mut child = &mut self.children[idx];


        while !child.is_leaf {
            let last = child.children.len() - 1;
            
            if child.children[last].keys.len() == T - 1 {
                if last > 0 && child.children[last - 1].keys.len() >= T {
                    child.borrow_left(last);
                } else {
                    child.merge_children(last - 1);
                }
            }
            let len = child.children.len()-1;
            child = &mut child.children[len];
            child.size -= 1;
        }
        return child.keys.pop().unwrap();
    }
    fn get_successor(&self, idx: usize) -> &(K, V) {
        let mut n = &self.children[idx + 1];
        while !n.is_leaf {
            n = &n.children.first().unwrap()
        }
        return n.keys.first().unwrap();
    }
    fn pop_successor(&mut self, idx: usize) -> (K, V) {
        let mut child = &mut self.children[idx + 1];

        while !child.is_leaf {
            if child.children[0].keys.len() == T - 1 {
                if child.children[1].keys.len() >= T {
                    child.borrow_right(0);
                } else {
                    child.merge_children(0);
                }
            }
            child = &mut child.children[0];
            child.size -= 1;
        }
        return child.keys.remove(0);
    }
    // fn print(&self, depth: usize) {
    //     let indent = "  ".repeat(depth);
    //
    //     print!("{}", indent);
    //     print!("[");
    //     for (i, k) in self.keys.iter().enumerate() {
    //         if i > 0 {
    //             print!(" ");
    //         }
    //         print!("{}", k);
    //     }
    //     println!("]");
    //
    //     if !self.is_leaf {
    //         for (i, child) in self.children.iter().enumerate() {
    //             print!("{}  ({}) 'c:{}'", indent, i, self.children[i].keys_len);
    //             child.print(depth + 1);
    //         }
    //     }
    // }
    //
    // fn validate(&self, is_root: bool) {
    //     // Check key bounds
    //     if !is_root {
    //         assert!(
    //             self.keys.len() >= T - 1,
    //             "Node underflow: {} keys",
    //             self.keys.len()
    //         );
    //     }
    //
    //     assert!(
    //         self.keys.len() <= 2 * T - 1,
    //         "Node overflow: {} keys",
    //         self.keys.len()
    //     );
    //
    //     // Check sorted keys
    //     for i in 1..self.keys.len() {
    //         assert!(self.keys[i - 1] < self.keys[i]);
    //     }
    //
    //     if !self.is_leaf {
    //         assert_eq!(
    //             self.children.len(),
    //             self.keys.len() + 1,
    //             "Children mismatch"
    //         );
    //
    //         for child in &self.children {
    //             child.validate(false);
    //         }
    //     }
    // }
}

impl<K: Ord, V> MarTree<K, V> {
    fn insert(&mut self, key: K, value: V) {
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
            new_root.insert_non_full(key, value);
            self.root = new_root;
        } else {
            self.root.insert_non_full(key, value);
        }
    }
    fn remove(&mut self, key: &K) {
        self.root.remove(key);
        if self.root.keys.is_empty() && !self.root.is_leaf {
            self.root = self.root.children.remove(0);
        }
    }
    // fn validate(&self) {
    //     self.root.validate(true);
    // }
    // fn print(&self, depth: usize) {
    //     self.root.print(depth);
    // }
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
        my_tree.insert(v, 0);
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
    println!(
        "Custom B-tree total keys: {} and size {}",
        my_tree.root.total_keys(),
        my_tree.root.size
    );

    // --- Test other's standard BTreeMap ---
    let mut itm_tree = IndexTreeSet::new();

    let start = Instant::now();
    for &v in &values {
        itm_tree.insert(v);
    }
    let duration = start.elapsed();
    println!(
        "IndexTreeSet insertion of {} elements took: {:?}",
        N, duration
    );

    // --- Benchmark deletion for custom B-tree ---

    let mut values_to_delete = values.clone();
    values_to_delete.shuffle(&mut rng);

    let start = Instant::now();
    for &v in &values_to_delete {
        my_tree.remove(&v);
    }
    let duration = start.elapsed();

    println!(
        "Custom B-tree deletion of {} elements took: {:?}",
        N, duration
    );

    println!(
        "Custom B-tree total keys after deletion: {} and by size: {}",
        my_tree.root.total_keys(),
        my_tree.root.size
    );

    // --- Benchmark deletion for std::BTreeSet ---

    let mut std_values = values.clone();
    std_values.shuffle(&mut rng);

    let start = Instant::now();
    for &v in &std_values {
        std_tree.remove(&v);
    }
    let duration = start.elapsed();

    println!(
        "std::BTreeSet deletion of {} elements took: {:?}",
        N, duration
    );

    println!("std::BTreeSet size after deletion: {}", std_tree.len());

    // // --- Benchmark deletion for indextreemap::indextree ---
    //
    // let start = Instant::now();
    // for &v in &std_values {
    //     itm_tree.remove(&v);
    // }
    // let duration = start.elapsed();
    //
    // println!(
    //     "IndexTreeSet deletion of {} elements took: {:?}",
    //     N, duration
    // );

    // println!("IndexTreeSet size after deletion: {}", itm_tree.len());
}
