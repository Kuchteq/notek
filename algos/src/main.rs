use std::{collections::BTreeSet, time::Instant};

use indextreemap::{IndexTreeMap, IndexTreeSet};
use rand::{rng, seq::SliceRandom};

trait Measured {
    fn measured(&self) -> usize;
}

struct Node<K, V>
where
    K: Ord,
{
    keys: Vec<(K, V)>,
    children: Vec<Node<K, V>>,
    size: usize,
    size_alt: usize,
    is_leaf: bool,
}

struct MarTree<K, V>
where
    K: Ord,
{
    root: Node<K, V>,
}

impl<K: Ord, V: Measured> Default for MarTree<K, V> {
    fn default() -> Self {
        MarTree {
            root: Node {
                keys: Vec::new(),
                children: Vec::new(),
                size: 0,
                size_alt: 0,
                is_leaf: true,
            },
        }
    }
}

const T: usize = 6;

impl<K: Ord, V: Measured> Default for Node<K, V> {
    fn default() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
            size: 0,
            size_alt: 0,
            is_leaf: true,
        }
    }
}

impl<K: Ord, V: Measured> Node<K, V> {
    // The function name is a precondition:
    // Insert key into the subtree rooted at node, assuming node itself is not full
    fn insert_non_full(&mut self, key: K, value: V) -> bool {
        // Read the binary_search_by documentation that states what the Err means
        match self.keys.binary_search_by(|(k, _)| k.cmp(&key)) {
            Ok(pos) => {
                let old_m = self.keys[pos].1.measured();
                let new_m = value.measured();
                self.keys[pos].1 = value;
                // Value changed — adjust size_alt for the difference
                self.size_alt = self.size_alt + new_m - old_m;
                false
            }
            Err(mut idx) => {
                if self.is_leaf {
                    let m = value.measured();
                    self.keys.insert(idx, (key, value));
                    self.size += 1;
                    self.size_alt += m;
                    true
                } else {
                    if self.children[idx].keys.len() == 2 * T - 1 {
                        self.split_child(idx);

                        // After split, the median was promoted to self.keys[idx].
                        // Check if the key we're inserting IS that median.
                        if key == self.keys[idx].0 {
                            let old_m = self.keys[idx].1.measured();
                            let new_m = value.measured();
                            self.keys[idx].1 = value;
                            self.size_alt = self.size_alt + new_m - old_m;
                            return false;
                        }

                        // After split, decide correct child
                        if key > self.keys[idx].0 {
                            idx += 1;
                        }
                    }

                    let old_child_alt = self.children[idx].size_alt;
                    let inserted = self.children[idx].insert_non_full(key, value);
                    let new_child_alt = self.children[idx].size_alt;
                    self.size_alt = self.size_alt + new_child_alt - old_child_alt;
                    if inserted {
                        self.size += 1;
                    }
                    inserted
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

    fn recompute_size_alt(&self) -> usize {
        let local: usize = self.keys.iter().map(|(_, v)| v.measured()).sum();
        if self.is_leaf {
            local
        } else {
            local + self.children.iter().map(|c| c.size_alt).sum::<usize>()
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
            size_alt: 0,
            is_leaf,
        };
        // Insert median key and new child into parent
        self.keys.insert(i, mid);
        self.children.insert(i + 1, right);
        self.children[i].size = self.children[i].recompute_size();
        self.children[i].size_alt = self.children[i].recompute_size_alt();
        self.children[i + 1].size = self.children[i + 1].recompute_size();
        self.children[i + 1].size_alt = self.children[i + 1].recompute_size_alt();
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
                let removed = self.keys.remove(idx);
                self.size -= 1;
                self.size_alt -= removed.1.measured();
                return true;
            }

            // 1B: Internal node
            if self.children[idx].keys.len() >= T {
                // The key being replaced has measured value old_m.
                // The predecessor that replaces it has measured value pred_m.
                // Net change at this node's keys: pred_m - old_m.
                // The child loses pred_m from its subtree.
                // So this node's size_alt changes by: (pred_m - old_m) - pred_m = -old_m.
                let old_m = self.keys[idx].1.measured();
                let old_child_alt = self.children[idx].size_alt;
                let pred = self.pop_predecessor(idx);
                let new_child_alt = self.children[idx].size_alt;
                let pred_m = pred.1.measured();
                self.keys[idx] = pred;
                self.size -= 1;
                // size_alt: lost old key's measured, gained pred's measured in keys,
                // but child lost pred's measured (captured by child_alt diff)
                self.size_alt = self.size_alt - old_m + pred_m - (old_child_alt - new_child_alt);
                return true;
            } else if self.children[idx + 1].keys.len() >= T {
                let old_m = self.keys[idx].1.measured();
                let old_child_alt = self.children[idx + 1].size_alt;
                let succ = self.pop_successor(idx);
                let new_child_alt = self.children[idx + 1].size_alt;
                let succ_m = succ.1.measured();
                self.keys[idx] = succ;
                self.size -= 1;
                self.size_alt = self.size_alt - old_m + succ_m - (old_child_alt - new_child_alt);
                return true;
            } else {
                // Merge: the key at self.keys[idx] moves into the child,
                // then we recursively remove from the child.
                // merge_children handles size_alt for the merge itself.
                // We track child size_alt before/after the recursive remove.
                let old_m = self.keys[idx].1.measured();
                self.merge_children(idx);
                let old_child_alt = self.children[idx].size_alt;
                let deleted = self.children[idx].remove(key);
                let new_child_alt = self.children[idx].size_alt;
                if deleted {
                    self.size -= 1;
                    // The merge moved old_m from our keys into the child (already accounted
                    // for by merge_children adjusting our size_alt via removing the key).
                    // The child's recursive remove changed child size_alt.
                    self.size_alt -= old_child_alt - new_child_alt;
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

        let old_child_alt = self.children[idx].size_alt;
        let deleted = self.children[idx].remove(key);
        let new_child_alt = self.children[idx].size_alt;
        if deleted {
            self.size -= 1;
            self.size_alt -= old_child_alt - new_child_alt;
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
        let borrowed_m = borrowed_key.1.measured();

        // 2 Swap with parent key
        let parent_key = std::mem::replace(&mut self.keys[idx - 1], borrowed_key);
        let parent_m = parent_key.1.measured();

        // 3 Insert old parent key into right child
        right.keys.insert(0, parent_key);

        // Size adjustments for key movement
        left.size -= 1;
        left.size_alt -= borrowed_m;
        right.size += 1;
        right.size_alt += parent_m;

        if !left.is_leaf {
            let moved_child = left.children.pop().unwrap();
            let moved_size = moved_child.size;
            let moved_alt = moved_child.size_alt;

            right.children.insert(0, moved_child);

            // Adjust sizes for subtree movement
            left.size -= moved_size;
            left.size_alt -= moved_alt;
            right.size += moved_size;
            right.size_alt += moved_alt;
        }

        // Parent's size_alt: self.keys changed from parent_m -> borrowed_m,
        // but children's total size_alt changed by (parent_m - borrowed_m),
        // so net change to self.size_alt is 0.
    }
    // borrows *from* right child to left one
    fn borrow_right(&mut self, idx: usize) {
        let (left_slice, right_slice) = self.children.split_at_mut(idx + 1);

        let left = &mut left_slice[idx];
        let right = &mut right_slice[0];

        let borrowed_key = right.keys.remove(0);
        let borrowed_m = borrowed_key.1.measured();
        let parent_key = std::mem::replace(&mut self.keys[idx], borrowed_key);
        let parent_m = parent_key.1.measured();
        left.keys.push(parent_key);

        left.size += 1;
        left.size_alt += parent_m;
        right.size -= 1;
        right.size_alt -= borrowed_m;

        if !right.is_leaf {
            let moved_child = right.children.remove(0);
            let moved_size = moved_child.size;
            let moved_alt = moved_child.size_alt;

            left.children.push(moved_child);

            left.size += moved_size;
            left.size_alt += moved_alt;
            right.size -= moved_size;
            right.size_alt -= moved_alt;
        }

        // Parent's size_alt has net zero change:
        // key-level: +borrowed_m - parent_m
        // children: +parent_m - borrowed_m (left gained parent_m, right lost borrowed_m)
        // Total: 0
    }

    fn merge_children(&mut self, idx: usize) {
        let right = self.children.remove(idx + 1);
        let child = &mut self.children[idx];

        let separator = self.keys.remove(idx);
        let sep_m = separator.1.measured();
        child.keys.push(separator);
        child.keys.extend(right.keys);

        if !child.is_leaf {
            child.children.extend(right.children);
        }

        child.size += right.size + 1;
        child.size_alt += right.size_alt + sep_m;
        // Parent's size_alt is UNCHANGED by merge:
        // - Lost sep_m from local keys
        // - Lost right.size_alt from children
        // - But left child gained (sep_m + right.size_alt)
        // Net effect: 0
    }

    fn get_predecessor(&self, idx: usize) -> &(K, V) {
        let mut n = &self.children[idx];
        while !n.is_leaf {
            n = &n.children.last().unwrap();
        }
        return n.keys.last().unwrap();
    }
    fn pop_predecessor(&mut self, idx: usize) -> (K, V) {
        // Pre-read the measured value of the predecessor we're about to pop
        let pred_m = self.get_predecessor(idx).1.measured();

        let mut child = &mut self.children[idx];

        while !child.is_leaf {
            child.size -= 1;
            child.size_alt -= pred_m;
            let last = child.children.len() - 1;

            if child.children[last].keys.len() == T - 1 {
                if last > 0 && child.children[last - 1].keys.len() >= T {
                    child.borrow_left(last);
                } else {
                    child.merge_children(last - 1);
                }
            }
            let len = child.children.len() - 1;
            child = &mut child.children[len];
        }
        child.size -= 1;
        child.size_alt -= pred_m;
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
        // Pre-read the measured value of the successor we're about to pop
        let succ_m = self.get_successor(idx).1.measured();

        let mut child = &mut self.children[idx + 1];

        while !child.is_leaf {
            child.size -= 1;
            child.size_alt -= succ_m;
            if child.children[0].keys.len() == T - 1 {
                if child.children.len() > 1 && child.children[1].keys.len() >= T {
                    child.borrow_right(0);
                } else {
                    child.merge_children(0);
                }
            }
            child = &mut child.children[0];
        }
        child.size -= 1;
        child.size_alt -= succ_m;
        return child.keys.remove(0);
    }
    fn validate(&self, is_root: bool) {
        // Check key count bounds
        if !is_root {
            assert!(
                self.keys.len() >= T - 1,
                "Node underflow: {} keys (min {})",
                self.keys.len(),
                T - 1
            );
        }
        assert!(
            self.keys.len() <= 2 * T - 1,
            "Node overflow: {} keys (max {})",
            self.keys.len(),
            2 * T - 1
        );

        // Check keys are strictly sorted
        for i in 1..self.keys.len() {
            assert!(
                self.keys[i - 1].0 < self.keys[i].0,
                "Keys not sorted at index {}",
                i
            );
        }

        let local_alt: usize = self.keys.iter().map(|(_, v)| v.measured()).sum();

        if self.is_leaf {
            // Leaf nodes must have no children
            assert!(
                self.children.is_empty(),
                "Leaf node has {} children",
                self.children.len()
            );
            // Leaf size must equal key count
            assert_eq!(
                self.size,
                self.keys.len(),
                "Leaf size {} != keys.len() {}",
                self.size,
                self.keys.len()
            );
            // Leaf size_alt must equal sum of measured values
            assert_eq!(
                self.size_alt, local_alt,
                "Leaf size_alt {} != sum of measured {}",
                self.size_alt, local_alt
            );
        } else {
            // Internal node must have exactly keys.len() + 1 children
            assert_eq!(
                self.children.len(),
                self.keys.len() + 1,
                "Children count {} != keys.len() + 1 = {}",
                self.children.len(),
                self.keys.len() + 1
            );

            // Size must equal keys.len() + sum of children sizes
            let expected_size =
                self.keys.len() + self.children.iter().map(|c| c.size).sum::<usize>();
            assert_eq!(
                self.size,
                expected_size,
                "Internal node size {} != expected {} (keys={}, children_sum={})",
                self.size,
                expected_size,
                self.keys.len(),
                self.children.iter().map(|c| c.size).sum::<usize>()
            );

            // size_alt must equal local keys' measured + sum of children size_alts
            let expected_alt = local_alt + self.children.iter().map(|c| c.size_alt).sum::<usize>();
            assert_eq!(
                self.size_alt,
                expected_alt,
                "Internal node size_alt {} != expected {} (local_alt={}, children_alt_sum={})",
                self.size_alt,
                expected_alt,
                local_alt,
                self.children.iter().map(|c| c.size_alt).sum::<usize>()
            );

            // All children must have the same leaf depth (checked via is_leaf consistency)
            // and all children must be non-root
            for child in &self.children {
                child.validate(false);
            }
        }

        // Size must also match the recursive total_keys count
        assert_eq!(
            self.size,
            self.total_keys(),
            "size {} != total_keys() {}",
            self.size,
            self.total_keys()
        );
    }
}

impl<K: Ord, V: Measured> MarTree<K, V> {
    fn insert(&mut self, key: K, value: V) {
        if self.root.keys.len() >= T * 2 - 1 {
            let old_root = std::mem::take(&mut self.root);
            let s = old_root.recompute_size();
            let s_alt = old_root.recompute_size_alt();
            let mut new_root = Node {
                keys: Vec::new(),
                children: vec![old_root],
                size: s,
                size_alt: s_alt,
                is_leaf: false,
            };
            new_root.split_child(0);
            new_root.insert_non_full(key, value);
            self.root = new_root;
        } else {
            self.root.insert_non_full(key, value);
        }
    }
    fn remove(&mut self, key: &K) -> bool {
        let removed = self.root.remove(key);
        if self.root.keys.is_empty() && !self.root.is_leaf {
            self.root = self.root.children.remove(0);
        }
        removed
    }
    fn validate(&self) {
        self.root.validate(true);
    }
    // fn print(&self, depth: usize) {
    //     self.root.print(depth);
    // }
}

impl Measured for usize {
    fn measured(&self) -> usize {
        1
    }
}

fn main() {
    const N: usize = 100_000; // number of elements to insert

    // Generate a sequence of numbers (you can shuffle for random order)
    let mut values: Vec<usize> = (0..N).collect();

    let mut rng = rng();

    values.shuffle(&mut rng);
    // --- Test your custom BTree ---
    let mut my_tree: MarTree<usize, usize> = MarTree::default();

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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rng, seq::SliceRandom, Rng};
    use std::collections::BTreeMap;

    impl Measured for i64 {
        fn measured(&self) -> usize {
            // Use absolute value as the "measurement" so tests exercise
            // non-trivial, varying weights.
            self.unsigned_abs() as usize
        }
    }

    fn new_tree() -> MarTree<i64, i64> {
        MarTree::default()
    }

    // -------------------------------------------------------
    // Basic operations
    // -------------------------------------------------------

    #[test]
    fn empty_tree() {
        let tree = new_tree();
        assert_eq!(tree.root.size, 0);
        assert_eq!(tree.root.total_keys(), 0);
        assert!(tree.root.is_leaf);
        assert!(tree.root.keys.is_empty());
        tree.validate();
    }

    #[test]
    fn single_insert_and_remove() {
        let mut tree = new_tree();
        tree.insert(42, 1);
        tree.validate();
        assert_eq!(tree.root.size, 1);
        assert_eq!(tree.root.keys[0], (42, 1));

        assert!(tree.remove(&42));
        tree.validate();
        assert_eq!(tree.root.size, 0);
        assert_eq!(tree.root.total_keys(), 0);
    }

    #[test]
    fn remove_nonexistent_key() {
        let mut tree = new_tree();
        for i in 0..20 {
            tree.insert(i * 2, 0); // insert evens
        }
        tree.validate();
        let size_before = tree.root.size;

        // Try removing odd numbers that don't exist
        assert!(!tree.remove(&1));
        assert!(!tree.remove(&3));
        assert!(!tree.remove(&99));
        assert!(!tree.remove(&-1));

        assert_eq!(tree.root.size, size_before);
        tree.validate();
    }

    #[test]
    fn remove_from_empty_tree() {
        let mut tree = new_tree();
        assert!(!tree.remove(&0));
        assert_eq!(tree.root.size, 0);
        tree.validate();
    }

    // -------------------------------------------------------
    // Duplicate key handling
    // -------------------------------------------------------

    #[test]
    fn duplicate_insert_updates_value_not_size() {
        let mut tree = new_tree();

        tree.insert(10, 100);
        tree.insert(20, 200);
        tree.insert(30, 300);
        assert_eq!(tree.root.size, 3);
        tree.validate();

        // Re-insert same key with different value
        tree.insert(20, 999);
        assert_eq!(tree.root.size, 3); // size must not change
        tree.validate();

        // Value should be updated — find it in the keys
        let found = tree.root.keys.iter().find(|(k, _)| *k == 20);
        assert_eq!(found, Some(&(20, 999)));
    }

    #[test]
    fn duplicate_insert_deep() {
        // Insert enough to force a multi-level tree, then re-insert a key
        // that lives deep in the tree.
        let mut tree = new_tree();
        for i in 0..500 {
            tree.insert(i, i);
        }
        tree.validate();
        assert_eq!(tree.root.size, 500);

        // Re-insert a key that is definitely not in the root node
        tree.insert(7, 9999);
        assert_eq!(tree.root.size, 500); // size unchanged
        tree.validate();
    }

    // -------------------------------------------------------
    // Sequential / ordered inserts
    // -------------------------------------------------------

    #[test]
    fn insert_ascending() {
        let mut tree = new_tree();
        let n = 500;
        for i in 0..n {
            tree.insert(i, i);
            tree.validate();
        }
        assert_eq!(tree.root.size, n as usize);
        assert_eq!(tree.root.total_keys(), n as usize);
    }

    #[test]
    fn insert_descending() {
        let mut tree = new_tree();
        let n = 500;
        for i in (0..n).rev() {
            tree.insert(i, i);
            tree.validate();
        }
        assert_eq!(tree.root.size, n as usize);
    }

    // -------------------------------------------------------
    // Random inserts with validation after each op
    // -------------------------------------------------------

    #[test]
    fn random_insert_validate_each() {
        let mut tree = new_tree();
        let mut rng = rng();
        let n = 300;
        let mut keys: Vec<i64> = (0..n).collect();
        keys.shuffle(&mut rng);

        for &k in &keys {
            tree.insert(k, k * 10);
            tree.validate();
        }
        assert_eq!(tree.root.size, n as usize);
    }

    // -------------------------------------------------------
    // Insert all, then remove all
    // -------------------------------------------------------

    #[test]
    fn insert_all_remove_all_sequential() {
        let mut tree = new_tree();
        let n = 500;
        for i in 0..n {
            tree.insert(i, 0);
        }
        tree.validate();

        for i in 0..n {
            assert!(tree.remove(&i));
            tree.validate();
        }
        assert_eq!(tree.root.size, 0);
        assert_eq!(tree.root.total_keys(), 0);
    }

    #[test]
    fn insert_all_remove_all_random_order() {
        let mut tree = new_tree();
        let mut rng = rng();
        let n = 1000;
        let mut keys: Vec<i64> = (0..n).collect();
        keys.shuffle(&mut rng);

        for &k in &keys {
            tree.insert(k, 0);
        }
        tree.validate();
        assert_eq!(tree.root.size, n as usize);

        keys.shuffle(&mut rng);
        for &k in &keys {
            assert!(tree.remove(&k));
            tree.validate();
        }
        assert_eq!(tree.root.size, 0);
        assert_eq!(tree.root.total_keys(), 0);
    }

    #[test]
    fn insert_ascending_delete_descending() {
        let mut tree = new_tree();
        let n: i64 = 500;
        for i in 0..n {
            tree.insert(i, 0);
        }
        tree.validate();

        for i in (0..n).rev() {
            assert!(tree.remove(&i));
            tree.validate();
        }
        assert_eq!(tree.root.size, 0);
    }

    #[test]
    fn insert_descending_delete_ascending() {
        let mut tree = new_tree();
        let n: i64 = 500;
        for i in (0..n).rev() {
            tree.insert(i, 0);
        }
        tree.validate();

        for i in 0..n {
            assert!(tree.remove(&i));
            tree.validate();
        }
        assert_eq!(tree.root.size, 0);
    }

    // -------------------------------------------------------
    // Interleaved insert and remove
    // -------------------------------------------------------

    #[test]
    fn interleaved_insert_remove() {
        let mut tree = new_tree();
        let mut reference = BTreeMap::new();
        let mut rng = rng();

        for _ in 0..2000 {
            let key: i64 = rng.random_range(0..200);
            if rng.random_bool(0.6) {
                // Insert
                tree.insert(key, key * 3);
                reference.insert(key, key * 3);
            } else {
                // Remove
                let tree_removed = tree.remove(&key);
                let ref_removed = reference.remove(&key).is_some();
                assert_eq!(
                    tree_removed, ref_removed,
                    "Mismatch on removing key {}",
                    key
                );
            }
            tree.validate();
            assert_eq!(
                tree.root.size,
                reference.len(),
                "Size mismatch: tree={} ref={}",
                tree.root.size,
                reference.len()
            );
        }
    }

    // -------------------------------------------------------
    // Size consistency invariant
    // -------------------------------------------------------

    #[test]
    fn size_equals_total_keys_throughout() {
        let mut tree = new_tree();
        let mut rng = rng();
        let mut expected_size: usize = 0;
        let mut present: std::collections::HashSet<i64> = std::collections::HashSet::new();

        for _ in 0..3000 {
            let key: i64 = rng.random_range(0..500);
            if rng.random_bool(0.55) {
                if present.insert(key) {
                    expected_size += 1;
                }
                tree.insert(key, 0);
            } else {
                if present.remove(&key) {
                    expected_size -= 1;
                }
                tree.remove(&key);
            }
            assert_eq!(tree.root.size, expected_size);
            assert_eq!(tree.root.total_keys(), expected_size);
        }
        tree.validate();
    }

    // -------------------------------------------------------
    // Partial removal then re-insert
    // -------------------------------------------------------

    #[test]
    fn partial_remove_then_reinsert() {
        let mut tree = new_tree();
        let n: i64 = 400;

        for i in 0..n {
            tree.insert(i, i);
        }
        tree.validate();

        // Remove the first half
        for i in 0..n / 2 {
            assert!(tree.remove(&i));
        }
        tree.validate();
        assert_eq!(tree.root.size, (n / 2) as usize);

        // Re-insert them
        for i in 0..n / 2 {
            tree.insert(i, i + 1000);
        }
        tree.validate();
        assert_eq!(tree.root.size, n as usize);

        // Remove everything
        for i in 0..n {
            assert!(tree.remove(&i));
        }
        tree.validate();
        assert_eq!(tree.root.size, 0);
    }

    // -------------------------------------------------------
    // Stress tests
    // -------------------------------------------------------

    #[test]
    fn stress_random_large() {
        let mut tree = new_tree();
        let mut reference = BTreeMap::new();
        let mut rng = rng();
        let ops = 10_000;

        for _ in 0..ops {
            let key: i64 = rng.random_range(0..2000);
            if rng.random_bool(0.55) {
                let val: i64 = rng.random_range(0..100_000);
                tree.insert(key, val);
                reference.insert(key, val);
            } else {
                let tree_removed = tree.remove(&key);
                let ref_removed = reference.remove(&key).is_some();
                assert_eq!(tree_removed, ref_removed);
            }
        }

        assert_eq!(tree.root.size, reference.len());
        assert_eq!(tree.root.total_keys(), reference.len());
        tree.validate();
    }

    #[test]
    fn stress_insert_remove_all_10k() {
        let mut tree = new_tree();
        let mut rng = rng();
        let n = 10_000;
        let mut keys: Vec<i64> = (0..n).collect();
        keys.shuffle(&mut rng);

        for &k in &keys {
            tree.insert(k, 0);
        }
        tree.validate();
        assert_eq!(tree.root.size, n as usize);

        keys.shuffle(&mut rng);
        for (i, &k) in keys.iter().enumerate() {
            assert!(tree.remove(&k), "Failed to remove key {} (op {})", k, i);
        }
        assert_eq!(tree.root.size, 0);
        assert_eq!(tree.root.total_keys(), 0);
        tree.validate();
    }

    // -------------------------------------------------------
    // Edge cases around T boundaries
    // -------------------------------------------------------

    #[test]
    fn insert_exactly_2t_minus_1_then_one_more() {
        // 2T-1 = 11 keys fit in a single root node; the 12th triggers a split
        let mut tree = new_tree();
        for i in 0..(2 * T as i64 - 1) {
            tree.insert(i, 0);
            tree.validate();
        }
        assert!(tree.root.is_leaf);
        assert_eq!(tree.root.keys.len(), 2 * T - 1);

        // One more forces root split
        tree.insert(2 * T as i64 - 1, 0);
        tree.validate();
        assert!(!tree.root.is_leaf);
        assert_eq!(tree.root.size, 2 * T);
    }

    #[test]
    fn remove_triggers_merge_and_borrow() {
        // Build a tree large enough that removals trigger both borrows and merges
        let mut tree = new_tree();
        let n: i64 = 200;
        for i in 0..n {
            tree.insert(i, 0);
        }
        tree.validate();

        // Remove from the middle — this exercises predecessor/successor replacement
        // and triggers rebalancing operations
        for i in (n / 4)..(3 * n / 4) {
            assert!(tree.remove(&i));
            tree.validate();
        }
        assert_eq!(tree.root.size, (n / 2) as usize);
    }

    #[test]
    fn double_remove_returns_false() {
        let mut tree = new_tree();
        for i in 0..50 {
            tree.insert(i, 0);
        }

        assert!(tree.remove(&25));
        assert!(!tree.remove(&25)); // already gone
        tree.validate();
        assert_eq!(tree.root.size, 49);
    }

    // -------------------------------------------------------
    // Leaf depth consistency
    // -------------------------------------------------------

    fn leaf_depths<K: Ord, V>(node: &Node<K, V>, depth: usize, out: &mut Vec<usize>) {
        if node.is_leaf {
            out.push(depth);
        } else {
            for child in &node.children {
                leaf_depths(child, depth + 1, out);
            }
        }
    }

    #[test]
    fn all_leaves_same_depth() {
        let mut tree = new_tree();
        let mut rng = rng();
        let n = 2000;
        let mut keys: Vec<i64> = (0..n).collect();
        keys.shuffle(&mut rng);

        for &k in &keys {
            tree.insert(k, 0);
        }

        let mut depths = Vec::new();
        leaf_depths(&tree.root, 0, &mut depths);
        let first = depths[0];
        assert!(
            depths.iter().all(|&d| d == first),
            "Not all leaves at same depth: {:?}",
            depths
        );

        // Remove half and check again
        keys.shuffle(&mut rng);
        for &k in &keys[..keys.len() / 2] {
            tree.remove(&k);
        }
        tree.validate();

        let mut depths = Vec::new();
        leaf_depths(&tree.root, 0, &mut depths);
        let first = depths[0];
        assert!(
            depths.iter().all(|&d| d == first),
            "After removals, not all leaves at same depth: {:?}",
            depths
        );
    }

    // -------------------------------------------------------
    // size_alt tracking tests
    // -------------------------------------------------------

    #[test]
    fn size_alt_empty_tree() {
        let tree = new_tree();
        assert_eq!(tree.root.size_alt, 0);
        tree.validate();
    }

    #[test]
    fn size_alt_single_element() {
        let mut tree = new_tree();
        tree.insert(1, 42);
        tree.validate();
        assert_eq!(tree.root.size_alt, 42);

        assert!(tree.remove(&1));
        tree.validate();
        assert_eq!(tree.root.size_alt, 0);
    }

    #[test]
    fn size_alt_accumulates_correctly() {
        let mut tree = new_tree();
        // Insert values 1..=100, each with measured() = value itself
        let mut expected_alt = 0usize;
        for i in 1..=100i64 {
            tree.insert(i, i);
            expected_alt += i as usize;
        }
        tree.validate();
        assert_eq!(tree.root.size_alt, expected_alt);
    }

    #[test]
    fn size_alt_update_on_duplicate_insert() {
        let mut tree = new_tree();
        tree.insert(1, 10);
        tree.insert(2, 20);
        tree.insert(3, 30);
        tree.validate();
        assert_eq!(tree.root.size_alt, 60); // 10 + 20 + 30

        // Update key 2's value from 20 to 50
        tree.insert(2, 50);
        tree.validate();
        assert_eq!(tree.root.size, 3); // count unchanged
        assert_eq!(tree.root.size_alt, 90); // 10 + 50 + 30
    }

    #[test]
    fn size_alt_update_on_deep_duplicate_insert() {
        let mut tree = new_tree();
        let mut expected_alt = 0usize;
        for i in 0..500i64 {
            tree.insert(i, i + 1); // measured = i+1
            expected_alt += (i + 1) as usize;
        }
        tree.validate();
        assert_eq!(tree.root.size_alt, expected_alt);

        // Update a deep key: key=7, old value=8, new value=1000
        tree.insert(7, 1000);
        expected_alt = expected_alt - 8 + 1000;
        tree.validate();
        assert_eq!(tree.root.size, 500);
        assert_eq!(tree.root.size_alt, expected_alt);
    }

    #[test]
    fn size_alt_after_removal() {
        let mut tree = new_tree();
        let mut expected_alt = 0usize;
        for i in 1..=200i64 {
            tree.insert(i, i);
            expected_alt += i as usize;
        }
        tree.validate();

        // Remove some elements
        for i in 1..=50i64 {
            assert!(tree.remove(&i));
            expected_alt -= i as usize;
            tree.validate();
            assert_eq!(tree.root.size_alt, expected_alt);
        }
    }

    #[test]
    fn size_alt_remove_nonexistent_unchanged() {
        let mut tree = new_tree();
        for i in 0..20i64 {
            tree.insert(i * 2, i * 2 + 1); // even keys, odd measured values
        }
        tree.validate();
        let alt_before = tree.root.size_alt;

        assert!(!tree.remove(&1));
        assert!(!tree.remove(&3));
        assert!(!tree.remove(&999));

        assert_eq!(tree.root.size_alt, alt_before);
        tree.validate();
    }

    #[test]
    fn size_alt_interleaved_insert_remove() {
        let mut tree = new_tree();
        let mut reference = BTreeMap::new();
        let mut rng = rng();

        for _ in 0..3000 {
            let key: i64 = rng.random_range(0..200);
            if rng.random_bool(0.6) {
                let val: i64 = rng.random_range(1..50);
                tree.insert(key, val);
                reference.insert(key, val);
            } else {
                tree.remove(&key);
                reference.remove(&key);
            }
            tree.validate();

            let expected_alt: usize = reference.values().map(|v| v.unsigned_abs() as usize).sum();
            assert_eq!(
                tree.root.size_alt, expected_alt,
                "size_alt mismatch: tree={} expected={}",
                tree.root.size_alt, expected_alt
            );
        }
    }

    #[test]
    fn size_alt_stress_random() {
        let mut tree = new_tree();
        let mut reference = BTreeMap::new();
        let mut rng = rng();

        for _ in 0..10_000 {
            let key: i64 = rng.random_range(0..2000);
            if rng.random_bool(0.55) {
                let val: i64 = rng.random_range(1..100);
                tree.insert(key, val);
                reference.insert(key, val);
            } else {
                tree.remove(&key);
                reference.remove(&key);
            }
        }

        tree.validate();
        let expected_alt: usize = reference.values().map(|v| v.unsigned_abs() as usize).sum();
        assert_eq!(tree.root.size_alt, expected_alt);
        assert_eq!(tree.root.size, reference.len());
    }

    // -------------------------------------------------------
    // Real-world use case: char values with byte-length measurement
    // -------------------------------------------------------

    /// A wrapper around char that measures its UTF-8 byte length.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Char(char);

    impl Measured for Char {
        fn measured(&self) -> usize {
            self.0.len_utf8()
        }
    }

    #[test]
    fn char_tree_byte_offsets() {
        let mut tree: MarTree<usize, Char> = MarTree::default();

        // Simulate a text buffer: index -> character
        let text = "Hello, 世界! 🌍";
        let chars: Vec<char> = text.chars().collect();

        let mut expected_bytes = 0usize;
        for (i, &ch) in chars.iter().enumerate() {
            tree.insert(i, Char(ch));
            expected_bytes += ch.len_utf8();
        }
        tree.validate();

        // size = number of characters
        assert_eq!(tree.root.size, chars.len());
        // size_alt = total byte length of the string
        assert_eq!(tree.root.size_alt, text.len());
        assert_eq!(tree.root.size_alt, expected_bytes);
    }

    #[test]
    fn char_tree_insert_remove_multibyte() {
        let mut tree: MarTree<usize, Char> = MarTree::default();

        // Insert a bunch of multi-byte characters
        let chars = vec!['a', 'é', '中', '🎉', 'b', 'ñ', '日', '🌈'];
        let mut expected_bytes = 0usize;
        for (i, &ch) in chars.iter().enumerate() {
            tree.insert(i, Char(ch));
            expected_bytes += ch.len_utf8();
            tree.validate();
            assert_eq!(tree.root.size_alt, expected_bytes);
        }

        // Remove some characters and verify byte count updates
        // Remove '中' (3 bytes)
        tree.remove(&2);
        expected_bytes -= '中'.len_utf8();
        tree.validate();
        assert_eq!(tree.root.size_alt, expected_bytes);

        // Remove '🎉' (4 bytes)
        tree.remove(&3);
        expected_bytes -= '🎉'.len_utf8();
        tree.validate();
        assert_eq!(tree.root.size_alt, expected_bytes);

        // Remove 'a' (1 byte)
        tree.remove(&0);
        expected_bytes -= 'a'.len_utf8();
        tree.validate();
        assert_eq!(tree.root.size_alt, expected_bytes);
    }

    #[test]
    fn char_tree_update_changes_byte_count() {
        let mut tree: MarTree<usize, Char> = MarTree::default();

        // Insert 'a' (1 byte)
        tree.insert(0, Char('a'));
        assert_eq!(tree.root.size_alt, 1);

        // Replace with '🌍' (4 bytes)
        tree.insert(0, Char('🌍'));
        tree.validate();
        assert_eq!(tree.root.size, 1); // still 1 character
        assert_eq!(tree.root.size_alt, 4); // but 4 bytes now
    }

    #[test]
    fn char_tree_stress_multibyte() {
        let mut tree: MarTree<usize, Char> = MarTree::default();
        let mut reference = BTreeMap::new();
        let mut rng = rng();

        // Pool of characters with different byte lengths
        let char_pool: Vec<char> = vec![
            'a', 'b', 'z', // 1 byte
            'é', 'ñ', 'ü', // 2 bytes
            '中', '日', '本', // 3 bytes
            '🌍', '🎉', '🚀', // 4 bytes
        ];

        for _ in 0..5000 {
            let key: usize = rng.random_range(0..500);
            if rng.random_bool(0.6) {
                let ch = char_pool[rng.random_range(0..char_pool.len())];
                tree.insert(key, Char(ch));
                reference.insert(key, Char(ch));
            } else {
                tree.remove(&key);
                reference.remove(&key);
            }
        }

        tree.validate();
        let expected_bytes: usize = reference.values().map(|c| c.0.len_utf8()).sum();
        assert_eq!(tree.root.size, reference.len());
        assert_eq!(tree.root.size_alt, expected_bytes);
    }
}
