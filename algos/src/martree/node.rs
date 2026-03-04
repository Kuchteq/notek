use std::cmp::Ordering;

use crate::martree::core::Measured;

#[derive(Debug, Clone)]
pub struct Node<K, V>
where
    K: Ord,
{
    pub keys: Vec<(K, V)>,
    pub children: Vec<Node<K, V>>,
    pub size: usize,
    pub size_alt: usize,
    pub is_leaf: bool,
}

pub const T: usize = 6;

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
    pub fn insert_non_full(&mut self, key: K, value: V) -> bool {
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
    pub fn recompute_size(&self) -> usize {
        if self.is_leaf {
            self.keys.len()
        } else {
            self.keys.len() + self.children.iter().map(|c| c.size).sum::<usize>()
        }
    }

    pub fn recompute_size_alt(&self) -> usize {
        let local: usize = self.keys.iter().map(|(_, v)| v.measured()).sum();
        if self.is_leaf {
            local
        } else {
            local + self.children.iter().map(|c| c.size_alt).sum::<usize>()
        }
    }

    pub fn split_child(&mut self, i: usize) {
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
    pub fn total_keys(&self) -> usize {
        let mut sum = self.keys.len();
        for child in &self.children {
            sum += child.total_keys();
        }
        sum
    }
    pub fn remove(&mut self, key: &K) -> bool {
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

    pub fn get(&self, key: &K) -> Option<&(K, V)> {
        let mut node = self;
        loop {
            match node.keys.binary_search_by(|(k, _)| k.cmp(&key)) {
                Ok(pos) => return Some(&node.keys[pos]),
                Err(pos) => {
                    if node.is_leaf {
                        return None;
                    }
                    node = &node.children[pos];
                }
            }
        }
    }

    pub fn get_next(&self, key: &K) -> Option<&(K, V)> {
        let mut node = self;
        let mut candidate = None;

        loop {
            match node.keys.binary_search_by(|(k, _)| k.cmp(key)) {
                Ok(pos) => {
                    if !node.is_leaf {
                        return Some(node.get_successor(pos));
                    }

                    if pos + 1 < node.keys.len() {
                        return Some(&node.keys[pos + 1]);
                    }

                    return candidate;
                }

                Err(pos) => {
                    if pos < node.keys.len() {
                        candidate = Some(&node.keys[pos]);
                    }

                    if node.is_leaf {
                        return candidate;
                    }

                    node = &node.children[pos];
                }
            }
        }
    }

    pub fn alt_to_index(&self, mut alt: usize) -> usize {
        let mut node = self;
        let mut i = 0;
        let mut idx = 0;
        if alt >= self.size_alt {
            return self.size;
        }
        loop {
            if node.is_leaf {
                for entry in &node.keys {
                    let m = entry.1.measured();
                    if alt < m {
                        return idx;
                    }
                    alt -= m;
                    idx += 1;
                }
                unreachable!("alt offset should be within leaf");
            }

            let child_alt = node.children[i].size_alt;
            if alt < child_alt {
                // Target is inside children[i]
                node = &node.children[i];
                i = 0;
            } else {
                alt -= child_alt;
                idx += node.children[i].size;
                let key_m = node.keys[i].1.measured();
                if alt < key_m {
                    // Target falls within keys[i]
                    return idx;
                }
                alt -= key_m;
                idx += 1;
                i += 1;
            }
        }
    }

    pub fn get_by_index(&self, mut idx: usize) -> Option<&(K, V)> {
        let mut node = self;
        let mut i = 0;
        if idx >= self.size {
            return None;
        }
        loop {
            if node.is_leaf {
                return Some(&node.keys[idx]);
            }

            match idx.cmp(&node.children[i].size) {
                Ordering::Less => {
                    node = &node.children[i];
                    i = 0;
                }
                Ordering::Equal => return Some(&node.keys[i]),
                Ordering::Greater => {
                    idx -= node.children[i].size + 1;
                    i += 1
                }
            }
        }
    }

    pub fn get_by_alt_size(&self, mut alt: usize) -> Option<&(K, V)> {
        let mut node = self;
        let mut i = 0;
        if alt >= self.size_alt {
            return None;
        }
        loop {
            if node.is_leaf {
                for entry in &node.keys {
                    let m = entry.1.measured();
                    if alt < m {
                        return Some(entry);
                    }
                    alt -= m;
                }
                unreachable!("alt offset should be within leaf");
            }

            let child_alt = node.children[i].size_alt;
            if alt < child_alt {
                // Target is inside children[i]
                node = &node.children[i];
                i = 0;
            } else {
                alt -= child_alt;
                let key_m = node.keys[i].1.measured();
                if alt < key_m {
                    // Target falls within keys[i]
                    return Some(&node.keys[i]);
                }
                alt -= key_m;
                i += 1;
            }
        }
    }

    pub fn validate(&self, is_root: bool) {
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
