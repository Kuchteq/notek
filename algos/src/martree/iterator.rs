use crate::martree::{MarTree, node::Node};

impl<K: Ord, V> MarTree<K, V> {
    pub fn iter(&self) -> Iter<'_, K, V> {
        let mut iter = Iter { stack: Vec::new() };
        iter.push_left(&self.root);
        iter
    }
}
pub struct Iter<'a, K: Ord, V> {
    stack: Vec<(&'a Node<K, V>, usize)>,
}

impl<'a, K:Ord, V> Iter<'a, K, V> {
    fn push_left(&mut self, mut node: &'a Node<K, V>) {
        loop {
            self.stack.push((node, 0));

            if node.is_leaf {
                break;
            }

            node = &node.children[0];
        }
    }
}
impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, idx)) = self.stack.pop() {

            if idx < node.keys.len() {
                // Push back node with incremented index
                self.stack.push((node, idx + 1));

                // If internal node, descend into right child
                if !node.is_leaf {
                    self.push_left(&node.children[idx + 1]);
                }

                return Some(&node.keys[idx]);
            }

            // else: node exhausted → continue popping
        }

        None
    }
}
