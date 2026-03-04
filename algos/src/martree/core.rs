use crate::martree::node::{Node, T};


pub trait Measured {
    fn measured(&self) -> usize;
}


#[derive(Debug, Clone)]
pub struct MarTree<K, V>
where
    K: Ord,
{
    pub(crate) root: Node<K, V>,
}

impl<K: Ord, V: Measured> Default for MarTree<K, V> {
    fn default() -> Self {
        MarTree {
            root: Node::default()
        }
    }
}


impl<K: Ord, V: Measured> MarTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
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
    pub fn remove(&mut self, key: &K) -> bool {
        let removed = self.root.remove(key);
        if self.root.keys.is_empty() && !self.root.is_leaf {
            self.root = self.root.children.remove(0);
        }
        removed
    }

    // TODO add options to remove by index/by size/by alt

    pub fn get(&self, key: &K) -> Option<&(K, V)> {
        self.root.get(key)
    }
    
    pub fn get_next(&self, key: &K) -> Option<&(K, V)> {
        self.root.get_next(key)
    }

    pub fn get_by_index(&self, idx: usize) -> Option<&(K, V)> {
        self.root.get_by_index(idx)
    }
    pub fn get_by_alt_size(&self, alt: usize) -> Option<&(K, V)> {
        self.root.get_by_alt_size(alt)
    }
    pub fn size(&self) -> usize {
        return self.root.size;
    }
    pub fn size_alt(&self) -> usize {
        return self.root.size_alt;
    }

    fn validate(&self) {
        self.root.validate(true);
    }
}
impl<K, V: Measured> FromIterator<(K, V)> for MarTree<K, V>
where
    K: Ord,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut tree = MarTree::default();

        for (k, v) in iter {
            tree.insert(k, v);
        }

        tree
    }
}




#[cfg(test)]
mod tests {
    use crate::martree::core::MarTree;

    use super::*;
    use rand::{Rng, rng, seq::SliceRandom};
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

    // -------------------------------------------------------
    // get() tests
    // -------------------------------------------------------

    #[test]
    fn get_empty_tree() {
        let tree = new_tree();
        assert!(tree.root.get(&0).is_none());
        assert!(tree.root.get(&42).is_none());
    }

    #[test]
    fn get_single_element() {
        let mut tree = new_tree();
        tree.insert(10, 100);
        assert_eq!(tree.root.get(&10), Some(&(10, 100)));
        assert!(tree.root.get(&9).is_none());
        assert!(tree.root.get(&11).is_none());
    }

    #[test]
    fn get_returns_updated_value_after_duplicate_insert() {
        let mut tree = new_tree();
        tree.insert(5, 50);
        assert_eq!(tree.root.get(&5), Some(&(5, 50)));
        tree.insert(5, 99);
        assert_eq!(tree.root.get(&5), Some(&(5, 99)));
    }

    #[test]
    fn get_after_removal() {
        let mut tree = new_tree();
        for i in 0..50 {
            tree.insert(i, i * 10);
        }
        assert_eq!(tree.root.get(&25), Some(&(25, 250)));
        tree.remove(&25);
        assert!(tree.root.get(&25).is_none());
        // Neighbors still there
        assert_eq!(tree.root.get(&24), Some(&(24, 240)));
        assert_eq!(tree.root.get(&26), Some(&(26, 260)));
    }

    #[test]
    fn get_all_inserted_keys() {
        let mut tree = new_tree();
        for i in 0..200 {
            tree.insert(i * 3, i);
        }
        tree.validate();
        for i in 0..200 {
            assert_eq!(tree.root.get(&(i * 3)), Some(&(i * 3, i)));
        }
        // Keys not inserted should be absent
        for i in 0..200 {
            let k = i * 3 + 1;
            assert!(tree.root.get(&k).is_none(), "key {} should not exist", k);
        }
    }

    #[test]
    fn get_deep_tree() {
        // Insert enough elements to create a multi-level tree
        let mut tree = new_tree();
        for i in 0..1000 {
            tree.insert(i, i * 2);
        }
        tree.validate();
        // Verify all keys
        for i in 0..1000 {
            assert_eq!(tree.root.get(&i), Some(&(i, i * 2)));
        }
        // Verify absent keys
        assert!(tree.root.get(&1000).is_none());
        assert!(tree.root.get(&-1).is_none());
    }

    // -------------------------------------------------------
    // get_by_index() tests
    // -------------------------------------------------------

    #[test]
    fn get_by_index_empty_tree() {
        let tree = new_tree();
        assert!(tree.root.get_by_index(0).is_none());
        assert!(tree.root.get_by_index(1).is_none());
    }

    #[test]
    fn get_by_index_single_element() {
        let mut tree = new_tree();
        tree.insert(42, 100);
        assert_eq!(tree.root.get_by_index(0), Some(&(42, 100)));
        assert!(tree.root.get_by_index(1).is_none());
    }

    #[test]
    fn get_by_index_returns_sorted_order() {
        let mut tree = new_tree();
        // Insert in scrambled order
        let keys = vec![50, 10, 30, 20, 40];
        for &k in &keys {
            tree.insert(k, k * 10);
        }
        tree.validate();
        // get_by_index should return in sorted key order
        assert_eq!(tree.root.get_by_index(0), Some(&(10, 100)));
        assert_eq!(tree.root.get_by_index(1), Some(&(20, 200)));
        assert_eq!(tree.root.get_by_index(2), Some(&(30, 300)));
        assert_eq!(tree.root.get_by_index(3), Some(&(40, 400)));
        assert_eq!(tree.root.get_by_index(4), Some(&(50, 500)));
        assert!(tree.root.get_by_index(5).is_none());
    }

    #[test]
    fn get_by_index_out_of_bounds() {
        let mut tree = new_tree();
        for i in 0..20 {
            tree.insert(i, 0);
        }
        assert!(tree.root.get_by_index(20).is_none());
        assert!(tree.root.get_by_index(100).is_none());
        assert!(tree.root.get_by_index(usize::MAX).is_none());
    }

    #[test]
    fn get_by_index_sequential_matches_sorted_keys() {
        let mut tree = new_tree();
        let mut rng = rng();
        let mut keys: Vec<i64> = (0..500).collect();
        keys.shuffle(&mut rng);
        for &k in &keys {
            tree.insert(k, k * 2);
        }
        tree.validate();

        keys.sort();
        for (idx, &k) in keys.iter().enumerate() {
            let result = tree.root.get_by_index(idx);
            assert_eq!(
                result,
                Some(&(k, k * 2)),
                "index {} expected key {} but got {:?}",
                idx,
                k,
                result
            );
        }
    }

    #[test]
    fn get_by_index_after_removals() {
        let mut tree = new_tree();
        for i in 0..100i64 {
            tree.insert(i, i);
        }
        // Remove even numbers
        for i in (0..100i64).step_by(2) {
            tree.remove(&i);
        }
        tree.validate();
        // Remaining: 1, 3, 5, 7, ..., 99 (50 elements)
        assert_eq!(tree.root.size, 50);
        for idx in 0..50 {
            let expected_key = (idx as i64) * 2 + 1;
            assert_eq!(
                tree.root.get_by_index(idx),
                Some(&(expected_key, expected_key)),
                "index {} should be key {}",
                idx,
                expected_key
            );
        }
        assert!(tree.root.get_by_index(50).is_none());
    }

    #[test]
    fn get_by_index_first_and_last() {
        let mut tree = new_tree();
        for i in 0..200i64 {
            tree.insert(i * 5, i);
        }
        tree.validate();
        // First element
        assert_eq!(tree.root.get_by_index(0), Some(&(0, 0)));
        // Last element
        assert_eq!(tree.root.get_by_index(199), Some(&(995, 199)));
    }

    #[test]
    fn get_by_index_stress_random_insert_remove() {
        let mut tree = new_tree();
        let mut reference: Vec<(i64, i64)> = Vec::new();
        let mut rng = rng();

        for _ in 0..5000 {
            let key: i64 = rng.random_range(0..500);
            if rng.random_bool(0.6) {
                let val = rng.random_range(0..1000);
                tree.insert(key, val);
                // Update reference (sorted vec)
                match reference.binary_search_by_key(&key, |&(k, _)| k) {
                    Ok(pos) => reference[pos].1 = val,
                    Err(pos) => reference.insert(pos, (key, val)),
                }
            } else {
                tree.remove(&key);
                if let Ok(pos) = reference.binary_search_by_key(&key, |&(k, _)| k) {
                    reference.remove(pos);
                }
            }
        }

        tree.validate();
        assert_eq!(tree.root.size, reference.len());

        // Verify every index matches reference
        for (idx, &(k, v)) in reference.iter().enumerate() {
            assert_eq!(
                tree.root.get_by_index(idx),
                Some(&(k, v)),
                "mismatch at index {}",
                idx
            );
        }
    }

    #[test]
    fn get_and_get_by_index_consistent() {
        // Verify that get_by_index and get agree: for every index,
        // the returned key should also be found by get()
        let mut tree = new_tree();
        let mut rng = rng();
        let mut keys: Vec<i64> = (0..300).collect();
        keys.shuffle(&mut rng);
        for &k in &keys {
            tree.insert(k, k + 1000);
        }
        tree.validate();

        for idx in 0..300 {
            let by_index = tree.root.get_by_index(idx).unwrap();
            let by_key = tree.root.get(&by_index.0).unwrap();
            assert_eq!(
                by_index, by_key,
                "get_by_index({}) and get({}) disagree",
                idx, by_index.0
            );
        }
    }

    // -------------------------------------------------------
    // get_by_alt_size() tests
    // -------------------------------------------------------

    #[test]
    fn get_by_alt_size_empty_tree() {
        let tree = new_tree();
        assert!(tree.root.get_by_alt_size(0).is_none());
        assert!(tree.root.get_by_alt_size(1).is_none());
    }

    #[test]
    fn get_by_alt_size_single_element() {
        let mut tree = new_tree();
        tree.insert(10, 5); // measured() = 5, occupies alt offsets [0..5)
        assert_eq!(tree.root.get_by_alt_size(0), Some(&(10, 5)));
        assert_eq!(tree.root.get_by_alt_size(4), Some(&(10, 5)));
        assert!(tree.root.get_by_alt_size(5).is_none());
    }

    #[test]
    fn get_by_alt_size_two_elements_varying_width() {
        let mut tree = new_tree();
        // key=1, value=3 → measured=3, alt offsets [0,1,2)
        // key=2, value=7 → measured=7, alt offsets [3,4,5,6,7,8,9)
        tree.insert(1, 3);
        tree.insert(2, 7);
        tree.validate();

        for alt in 0..3 {
            assert_eq!(
                tree.root.get_by_alt_size(alt),
                Some(&(1, 3)),
                "alt {} should map to key 1",
                alt
            );
        }
        for alt in 3..10 {
            assert_eq!(
                tree.root.get_by_alt_size(alt),
                Some(&(2, 7)),
                "alt {} should map to key 2",
                alt
            );
        }
        assert!(tree.root.get_by_alt_size(10).is_none());
    }

    #[test]
    fn get_by_alt_size_out_of_bounds() {
        let mut tree = new_tree();
        for i in 1..=20i64 {
            tree.insert(i, i); // measured = abs(i) = i
        }
        let total_alt: usize = (1..=20).sum::<usize>(); // 210
        assert_eq!(tree.root.size_alt, total_alt);
        assert!(tree.root.get_by_alt_size(total_alt).is_none());
        assert!(tree.root.get_by_alt_size(total_alt + 100).is_none());
        assert!(tree.root.get_by_alt_size(usize::MAX).is_none());
    }

    #[test]
    fn get_by_alt_size_uniform_width_matches_get_by_index() {
        // When all values have measured() = 1, get_by_alt_size(n)
        // should equal get_by_index(n)
        let mut tree = new_tree();
        for i in 0..200i64 {
            tree.insert(i, 1); // measured = 1
        }
        tree.validate();
        for idx in 0..200 {
            assert_eq!(
                tree.root.get_by_alt_size(idx),
                tree.root.get_by_index(idx),
                "with uniform measured=1, alt_size({}) should equal by_index({})",
                idx,
                idx
            );
        }
    }

    #[test]
    fn get_by_alt_size_sequential_scan() {
        // Insert keys with varying measured values,
        // then verify every alt offset maps to the correct entry.
        let mut tree = new_tree();
        let entries: Vec<(i64, i64)> = vec![
            (0, 2), // measured=2, alt [0,1)
            (1, 5), // measured=5, alt [2,3,4,5,6)
            (2, 1), // measured=1, alt [7)
            (3, 3), // measured=3, alt [8,9,10)
            (4, 4), // measured=4, alt [11,12,13,14)
        ];
        for &(k, v) in &entries {
            tree.insert(k, v);
        }
        tree.validate();
        assert_eq!(tree.root.size_alt, 15);

        // Build expected mapping: alt_offset -> (key, value)
        let mut expected: Vec<&(i64, i64)> = Vec::new();
        for entry in &entries {
            for _ in 0..entry.1.unsigned_abs() {
                expected.push(entry);
            }
        }

        for (alt, &exp) in expected.iter().enumerate() {
            assert_eq!(
                tree.root.get_by_alt_size(alt),
                Some(exp),
                "alt {} expected {:?}",
                alt,
                exp
            );
        }
    }

    #[test]
    fn get_by_alt_size_after_removal() {
        let mut tree = new_tree();
        // Insert keys 0..50 with measured = key+1 (so no zeros)
        for i in 0..50i64 {
            tree.insert(i, i + 1);
        }
        // Remove every other key
        for i in (0..50i64).step_by(2) {
            tree.remove(&i);
        }
        tree.validate();

        // Build reference: remaining entries sorted by key
        let remaining: Vec<(i64, i64)> = (0..50i64)
            .filter(|i| i % 2 != 0)
            .map(|i| (i, i + 1))
            .collect();
        let total_alt: usize = remaining
            .iter()
            .map(|(_, v)| v.unsigned_abs() as usize)
            .sum();
        assert_eq!(tree.root.size_alt, total_alt);

        // Scan all alt offsets
        let mut alt = 0;
        for &(k, v) in &remaining {
            let m = v.unsigned_abs() as usize;
            for offset in 0..m {
                assert_eq!(
                    tree.root.get_by_alt_size(alt + offset),
                    Some(&(k, v)),
                    "alt {} should be key {}",
                    alt + offset,
                    k
                );
            }
            alt += m;
        }
        assert!(tree.root.get_by_alt_size(alt).is_none());
    }

    #[test]
    fn get_by_alt_size_char_tree_byte_offsets() {
        // Use the Char type where measured() = len_utf8()
        let mut tree: MarTree<usize, Char> = MarTree::default();
        let text = "aé中🌍b";
        let chars: Vec<char> = text.chars().collect();
        // 'a'=1byte, 'é'=2bytes, '中'=3bytes, '🌍'=4bytes, 'b'=1byte
        for (i, &ch) in chars.iter().enumerate() {
            tree.insert(i, Char(ch));
        }
        tree.validate();
        assert_eq!(tree.root.size_alt, text.len()); // 1+2+3+4+1=11

        // alt offsets:
        // [0)       -> 'a'   (key=0)
        // [1,2)     -> 'é'   (key=1)
        // [3,4,5)   -> '中'  (key=2)
        // [6,7,8,9) -> '🌍' (key=3)
        // [10)      -> 'b'   (key=4)
        let expected: Vec<(usize, usize, Char)> = vec![
            (0, 1, Char('a')),
            (1, 2, Char('é')),
            (3, 3, Char('中')),
            (6, 4, Char('🌍')),
            (10, 1, Char('b')),
        ];
        for (start, width, ch) in &expected {
            for offset in 0..*width {
                let result = tree.root.get_by_alt_size(start + offset);
                assert!(
                    result.is_some(),
                    "alt {} should find a char",
                    start + offset
                );
                assert_eq!(
                    result.unwrap().1.0,
                    ch.0,
                    "alt {} should be '{}'",
                    start + offset,
                    ch.0
                );
            }
        }
        assert!(tree.root.get_by_alt_size(11).is_none());
    }

    #[test]
    fn get_by_alt_size_stress_random() {
        let mut tree = new_tree();
        let mut reference: Vec<(i64, i64)> = Vec::new();
        let mut rng = rng();

        // Insert random entries with non-zero measured values
        for _ in 0..2000 {
            let key: i64 = rng.random_range(0..500);
            let val: i64 = rng.random_range(1..10); // 1..10 so measured() >= 1
            if rng.random_bool(0.7) {
                tree.insert(key, val);
                match reference.binary_search_by_key(&key, |&(k, _)| k) {
                    Ok(pos) => reference[pos].1 = val,
                    Err(pos) => reference.insert(pos, (key, val)),
                }
            } else {
                tree.remove(&key);
                if let Ok(pos) = reference.binary_search_by_key(&key, |&(k, _)| k) {
                    reference.remove(pos);
                }
            }
        }

        tree.validate();
        let total_alt: usize = reference
            .iter()
            .map(|(_, v)| v.unsigned_abs() as usize)
            .sum();
        assert_eq!(tree.root.size_alt, total_alt);

        // Verify every alt offset maps to the correct entry
        let mut alt = 0usize;
        for &(k, v) in &reference {
            let m = v.unsigned_abs() as usize;
            // Check first and last offset within this entry's range
            assert_eq!(
                tree.root.get_by_alt_size(alt),
                Some(&(k, v)),
                "alt {} (start of key {}) mismatch",
                alt,
                k
            );
            if m > 1 {
                assert_eq!(
                    tree.root.get_by_alt_size(alt + m - 1),
                    Some(&(k, v)),
                    "alt {} (end of key {}) mismatch",
                    alt + m - 1,
                    k
                );
            }
            alt += m;
        }
        assert!(tree.root.get_by_alt_size(alt).is_none());
    }

    #[test]
    fn get_by_alt_size_char_tree_stress() {
        let mut tree: MarTree<usize, Char> = MarTree::default();
        let mut reference: BTreeMap<usize, Char> = BTreeMap::new();
        let mut rng = rng();

        let char_pool: Vec<char> = vec![
            'a', 'z', // 1 byte
            'é', 'ñ', // 2 bytes
            '中', '日', // 3 bytes
            '🌍', '🚀', // 4 bytes
        ];

        for _ in 0..3000 {
            let key: usize = rng.random_range(0..300);
            if rng.random_bool(0.65) {
                let ch = char_pool[rng.random_range(0..char_pool.len())];
                tree.insert(key, Char(ch));
                reference.insert(key, Char(ch));
            } else {
                tree.remove(&key);
                reference.remove(&key);
            }
        }

        tree.validate();
        let total_bytes: usize = reference.values().map(|c| c.0.len_utf8()).sum();
        assert_eq!(tree.root.size_alt, total_bytes);

        // Verify alt offsets
        let entries: Vec<(usize, Char)> = reference.into_iter().collect();
        let mut alt = 0usize;
        for &(k, ref ch) in &entries {
            let m = ch.0.len_utf8();
            for offset in 0..m {
                let result = tree.root.get_by_alt_size(alt + offset);
                assert_eq!(
                    result.map(|e| e.0),
                    Some(k),
                    "byte offset {} should map to key {}",
                    alt + offset,
                    k
                );
            }
            alt += m;
        }
        assert!(tree.root.get_by_alt_size(alt).is_none());
    }

    #[test]
    fn get_by_alt_size_boundary_between_entries() {
        // Specifically test the boundary where one entry ends and the next begins
        let mut tree = new_tree();
        tree.insert(1, 3); // alt [0,1,2)
        tree.insert(2, 5); // alt [3,4,5,6,7)
        tree.insert(3, 2); // alt [8,9)

        // Last offset of entry 1
        assert_eq!(tree.root.get_by_alt_size(2), Some(&(1, 3)));
        // First offset of entry 2
        assert_eq!(tree.root.get_by_alt_size(3), Some(&(2, 5)));
        // Last offset of entry 2
        assert_eq!(tree.root.get_by_alt_size(7), Some(&(2, 5)));
        // First offset of entry 3
        assert_eq!(tree.root.get_by_alt_size(8), Some(&(3, 2)));
        // Last offset of entry 3
        assert_eq!(tree.root.get_by_alt_size(9), Some(&(3, 2)));
        // Past the end
        assert!(tree.root.get_by_alt_size(10).is_none());
    }
}
