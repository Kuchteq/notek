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
        self.size += 1;
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
    fn remove(&mut self, val: usize) -> bool {
        let mut idx = 0;
        while idx < self.keys.len() && self.keys[idx] < val {
            idx += 1;
        }

        // -------------------------
        // CASE 1: Key found here
        // -------------------------
        if idx < self.keys.len() && self.keys[idx] == val {
            // 1A: Leaf
            if self.is_leaf {
                self.keys.remove(idx);
                self.size -= 1;
                return true;
            }

            // 1B: Internal node
            if self.children[idx].keys.len() >= T {
                let pred = self.get_predecessor(idx);
                self.keys[idx] = pred;

                let deleted = self.children[idx].remove(pred);
                if deleted {
                    self.size -= 1;
                }
                return deleted;
            } else if self.children[idx + 1].keys.len() >= T {
                let succ = self.get_successor(idx);
                self.keys[idx] = succ;

                let deleted = self.children[idx + 1].remove(succ);
                if deleted {
                    self.size -= 1;
                }
                return deleted;
            } else {
                self.merge_children(idx);
                let deleted = self.children[idx].remove(val);
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

        let deleted = self.children[idx].remove(val);
        if deleted {
            self.size -= 1;
        }
        deleted
    }
    fn borrow_left(&mut self, idx: usize) {
        // Move parent key into child
        let parent_key = self.keys[idx - 1];

        let (left_slice, right_slice) = self.children.split_at_mut(idx);

        let left = &mut left_slice[idx - 1];
        let right = &mut right_slice[0];

        right.keys.insert(0, parent_key);

        // Replace parent key with left's last key
        let borrowed_key = left.keys.pop().unwrap();
        self.keys[idx - 1] = borrowed_key;

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
    fn borrow_right(&mut self, idx: usize) {
        let parent_key = self.keys[idx];

        let (left_slice, right_slice) = self.children.split_at_mut(idx + 1);

        let left = &mut left_slice[idx];
        let right = &mut right_slice[0];

        left.keys.push(parent_key);

        let borrowed_key = right.keys.remove(0);
        self.keys[idx] = borrowed_key;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    // ------------------------------------------------------------
    // Helper: full structural + size invariant verification
    // ------------------------------------------------------------
    fn assert_full_invariants(node: &Node, is_root: bool) -> usize {
        // 1. Key bounds
        if !is_root {
            assert!(
                node.keys.len() >= T - 1,
                "Underflow: {} keys",
                node.keys.len()
            );
        }
        assert!(
            node.keys.len() <= 2 * T - 1,
            "Overflow: {} keys",
            node.keys.len()
        );

        // 2. Sorted keys
        for i in 1..node.keys.len() {
            assert!(node.keys[i - 1] < node.keys[i], "Keys not sorted");
        }

        // 3. Children count
        if node.is_leaf {
            assert!(node.children.is_empty(), "Leaf has children");
        } else {
            assert_eq!(
                node.children.len(),
                node.keys.len() + 1,
                "Children mismatch"
            );
        }

        // 4. Recursively compute real subtree size
        let mut actual_size = node.keys.len();

        for child in &node.children {
            actual_size += assert_full_invariants(child, false);
        }

        // 5. Size correctness
        assert_eq!(
            node.size, actual_size,
            "Size mismatch: stored={}, actual={}",
            node.size, actual_size
        );

        actual_size
    }

    // ------------------------------------------------------------
    // Insert sequential
    // ------------------------------------------------------------
    #[test]
    fn insert_sequential() {
        let mut tree = MarTree::default();

        for i in 0..1000 {
            tree.insert(i);
        }

        assert_eq!(tree.root.size, 1000);
        assert_full_invariants(&tree.root, true);
    }

    // ------------------------------------------------------------
    // Delete sequential
    // ------------------------------------------------------------
    #[test]
    fn delete_sequential() {
        let mut tree = MarTree::default();

        for i in 0..1000 {
            tree.insert(i);
        }

        for i in 0..1000 {
            tree.remove(i);
            assert_eq!(tree.root.size, 999 - i);
            assert_full_invariants(&tree.root, true);
        }

        assert_eq!(tree.root.size, 0);
        assert!(tree.root.is_leaf);
    }

    // ------------------------------------------------------------
    // Delete non-existent values
    // ------------------------------------------------------------
    #[test]
    fn delete_nonexistent() {
        let mut tree = MarTree::default();

        for i in 0..200 {
            tree.insert(i);
        }

        tree.remove(9999);

        assert_eq!(tree.root.size, 200);
        assert_full_invariants(&tree.root, true);
    }

    // ------------------------------------------------------------
    // Reverse insert then delete
    // ------------------------------------------------------------
    #[test]
    fn reverse_insert_delete() {
        let mut tree = MarTree::default();

        for i in (0..500).rev() {
            tree.insert(i);
        }

        assert_full_invariants(&tree.root, true);

        for i in (0..500).rev() {
            tree.remove(i);
            assert_full_invariants(&tree.root, true);
        }

        assert_eq!(tree.root.size, 0);
    }

    // ------------------------------------------------------------
    // Randomized reference test
    // ------------------------------------------------------------
    #[test]
    fn randomized_against_btreeset() {
        let mut tree = MarTree::default();
        let mut reference = BTreeSet::new();

        for i in 0..500 {
            tree.insert(i);
            reference.insert(i);
        }

        for i in (0..500).rev() {
            tree.remove(i);
            reference.remove(&i);

            assert_eq!(tree.root.size, reference.len());
            assert_full_invariants(&tree.root, true);
        }

        assert_eq!(tree.root.size, 0);
    }

    // ------------------------------------------------------------
    // Mixed random operations stress test
    // ------------------------------------------------------------
    #[test]
    fn stress_mixed_operations() {
        use rand::Rng;

        let mut tree = MarTree::default();
        let mut reference = BTreeSet::new();
        let mut rng = rand::thread_rng();

        for _ in 0..5000 {
            let value = rng.gen_range(0..1000);

            if rng.gen_bool(0.5) {
                let inserted = reference.insert(value);
                if inserted {
                    tree.insert(value);
                }
            } else {
                tree.remove(value);
                reference.remove(&value);
            }

            assert_eq!(tree.root.size, reference.len());
            assert_full_invariants(&tree.root, true);
        }
    }

    // ------------------------------------------------------------
    // Root shrink behavior
    // ------------------------------------------------------------
    #[test]
    fn root_shrinks_correctly() {
        let mut tree = MarTree::default();

        for i in 0..200 {
            tree.insert(i);
        }

        for i in 0..200 {
            tree.remove(i);
            assert_full_invariants(&tree.root, true);
        }

        assert_eq!(tree.root.size, 0);
        assert!(tree.root.is_leaf);
    }

    // ------------------------------------------------------------
    // Internal size consistency vs total_keys()
    // ------------------------------------------------------------
    #[test]
    fn size_matches_total_keys_function() {
        let mut tree = MarTree::default();

        for i in 0..300 {
            tree.insert(i);
        }

        assert_eq!(tree.root.size, tree.root.total_keys());

        for i in 0..300 {
            tree.remove(i);
            assert_eq!(tree.root.size, tree.root.total_keys());
        }
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

    // // --- Test GPT's BTreeMap ---
    // let mut std_tree = CountedBTreeMap::new();
    //
    // let start = Instant::now();
    // for &v in &values {
    //     std_tree.insert(v, ());
    // }
    // let duration = start.elapsed();
    // println!(
    //     "gpt::CountedBTreeMap insertion of {} elements took: {:?}",
    //     N, duration
    // );
    //
    // println!("gpt::CountedBTreeMap size: {}", std_tree.len());

    let mut visualizer = MarTree::default();
    visualizer.insert(11);
    visualizer.insert(2);
    visualizer.insert(3);
    visualizer.insert(4);
    visualizer.insert(1);
    visualizer.insert(8);
    visualizer.insert(9);
    visualizer.insert(18);
    visualizer.insert(0);
    visualizer.insert(8);
    visualizer.insert(8);
    visualizer.insert(4);
    visualizer.remove(0);
    visualizer.remove(1);
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
