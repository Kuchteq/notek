package dev.kuchta.notek.sync

import java.util.NoSuchElementException

/**
 * Balanced order-statistic tree map.
 *
 * Kotlin port and extension of the previous BalancedOrderStatisticTree to behave like
 * a map: comparable keys -> values (similar to java.util.TreeMap) while keeping
 * O(log n) rank/select via stored left/right subtree counts.
 */
class BalancedOrderStatisticTreeMap<K : Comparable<K>, V> : MutableMap<K, V> {

    inner class Node(var key: K, var value: V) {
        var parent: Node? = null
        var lChild: Node? = null
        var rChild: Node? = null
        var left: Int = 0
        var right: Int = 0
        var height: Int = 1

        fun head(): Node {
            var tmp: Node = this
            while (tmp.lChild != null) tmp = tmp.lChild!!
            return tmp
        }

        fun tail(): Node {
            var tmp: Node = this
            while (tmp.rChild != null) tmp = tmp.rChild!!
            return tmp
        }

        private fun heightLeft(): Int = lChild?.height ?: 0
        private fun heightRight(): Int = rChild?.height ?: 0

        private fun rotateLeftUp(rightHeight: Int) {
            check(lChild != null)

            val child = lChild!!
            val hLeft = child.heightLeft()
            val hRight = child.heightRight()

            lChild = child.rChild
            child.rChild = this

            child.parent = this.parent
            this.parent = child
            if (lChild != null) lChild!!.parent = this
            if (child.parent == null) {
                root = child
            } else {
                if (child.parent!!.lChild == this) {
                    child.parent!!.lChild = child
                } else {
                    check(child.parent!!.rChild == this)
                    child.parent!!.rChild = child
                }
            }

            this.left = child.right
            child.right = this.left + 1 + this.right

            this.height = if (hRight > rightHeight) hRight + 1 else rightHeight + 1
            child.height = if (hLeft > this.height) hLeft + 1 else this.height + 1

            assert(child.verify())
        }

        private fun rotateRightUp(leftHeight: Int) {
            check(rChild != null)

            val child = rChild!!
            val hLeft = child.heightLeft()
            val hRight = child.heightRight()

            rChild = child.lChild
            child.lChild = this

            child.parent = this.parent
            this.parent = child
            if (rChild != null) rChild!!.parent = this
            if (child.parent == null) {
                root = child
            } else {
                if (child.parent!!.lChild == this) {
                    child.parent!!.lChild = child
                } else {
                    check(child.parent!!.rChild == this)
                    child.parent!!.rChild = child
                }
            }

            this.right = child.left
            child.left = this.left + 1 + this.right

            this.height = if (hLeft > leftHeight) hLeft + 1 else leftHeight + 1
            child.height = if (hRight > this.height) hRight + 1 else this.height + 1

            assert(child.verify())
        }

        /**
         * Exposed so removal code in the outer class can call it.
         */
        fun updateHeight(): Boolean {
            val hLeft = heightLeft()
            val hRight = heightRight()
            if (hLeft > hRight + 1) {
                check(lChild != null)
                val hLLeft = lChild!!.heightLeft()
                val hLRight = lChild!!.heightRight()
                if (hLLeft < hLRight) {
                    lChild!!.rotateRightUp(hLLeft)
                }
                rotateLeftUp(hRight)
                parent?.height = 0
                return true
            } else if (hRight > hLeft + 1) {
                check(rChild != null)
                val hRLeft = rChild!!.heightLeft()
                val hRRight = rChild!!.heightRight()
                if (hRLeft > hRRight) {
                    rChild!!.rotateLeftUp(hRRight)
                }
                rotateRightUp(hLeft)
                parent?.height = 0
                return true
            }

            return if (hLeft > hRight) {
                if (height != hLeft + 1) {
                    height = hLeft + 1
                    true
                } else {
                    false
                }
            } else {
                if (height != hRight + 1) {
                    height = hRight + 1
                    true
                } else {
                    false
                }
            }
        }

        fun rank(): Int {
            var c = left
            var tmp: Node? = this
            while (tmp?.parent != null) {
                if (tmp.parent!!.lChild != tmp) {
                    c += tmp.parent!!.left + 1
                }
                tmp = tmp.parent
            }
            return c
        }

        fun select(index: Int): Node {
            return when {
                index == left -> this
                index < left -> lChild!!.select(index)
                else -> rChild!!.select(index - left - 1)
            }
        }

        private fun insert(n: Node) {
            val cmp = n.key.compareTo(key)
            if (cmp < 0) {
                left++
                if (lChild == null) {
                    n.parent = this
                    lChild = n
                    if (rChild == null) {
                        height++
                        var p = parent
                        while (p != null) {
                            if (!p.updateHeight()) break
                            p = p.parent
                        }
                    }
                } else {
                    lChild!!.insert(n)
                }
            } else if (cmp > 0) {
                right++
                if (rChild == null) {
                    n.parent = this
                    rChild = n
                    if (lChild == null) {
                        height++
                        var p = parent
                        while (p != null) {
                            if (!p.updateHeight()) break
                            p = p.parent
                        }
                    }
                } else {
                    rChild!!.insert(n)
                }
            } else {
                // equal key: replace value
                this.value = n.value
            }
        }

        fun next(): Node? {
            if (rChild != null) return rChild!!.head()
            if (parent == null) return null
            if (parent!!.lChild == this) return parent
            var tmp: Node? = this
            while (tmp?.parent != null && tmp.parent!!.rChild == tmp) tmp = tmp.parent
            return tmp?.parent
        }

        fun prev(): Node? {
            if (lChild != null) return lChild!!.tail()
            if (parent == null) return null
            if (parent!!.rChild == this) return parent
            var tmp: Node? = this
            while (tmp?.parent != null && tmp.parent!!.lChild == tmp) tmp = tmp.parent
            return tmp?.parent
        }

        fun findKey(k: K): Node? {
            val cmp = key.compareTo(k)
            return when {
                cmp == 0 -> this
                cmp < 0 -> rChild?.findKey(k)
                else -> lChild?.findKey(k)
            }
        }

        fun verify(): Boolean {
            var hLeft = 0
            var hRight = 0
            if (parent != null) {
                check(parent!!.lChild == this || parent!!.rChild == this) { "Parent-child link inconsistent for $key" }
            }
            if (lChild != null) {
                check(lChild!!.parent == this) { "Left child (${lChild!!.key}) has wrong parent" }
                lChild!!.verify()
                check(left == lChild!!.left + 1 + lChild!!.right) { "Left count mismatch at node $key" }
                hLeft = lChild!!.height
            } else {
                check(left == 0)
            }
            if (rChild != null) {
                check(rChild!!.parent == this) { "Right child (${rChild!!.key}) has wrong parent" }
                rChild!!.verify()
                check(right == rChild!!.left + 1 + rChild!!.right) { "Right count mismatch at node $key" }
                hRight = rChild!!.height
            } else {
                check(right == 0)
            }
            if (hLeft > hRight) {
                check(height == hLeft + 1) { "Height mismatch at node $key" }
            } else {
                check(height == hRight + 1) { "Height mismatch at node $key" }
            }
            return true
        }
    }

    private var root: Node? = null
    private var head: Node? = null
    private var tail: Node? = null

    // ---------- basic map operations ----------

    override val size: Int
        get() = root?.let { it.left + 1 + it.right } ?: 0

    override fun isEmpty(): Boolean = root == null

    override fun containsKey(key: K): Boolean = get(key) != null

    override fun containsValue(value: V): Boolean {
        // linear-time scan
        var n = head
        while (n != null) {
            if (n.value == value) return true
            n = n.next()
        }
        return false
    }

    override fun get(key: K): V? {
        val n = root?.findKey(key)
        return n?.value
    }

    override fun put(key: K, value: V): V? {
        val newNode = Node(key, value)
        if (root == null) {
            root = newNode
            head = newNode
            tail = newNode
            return null
        }

        // walk tree to insert or replace
        var cur = root
        while (cur != null) {
            val cmp = key.compareTo(cur.key)
            when {
                cmp < 0 -> {
                    cur.left++
                    if (cur.lChild == null) {
                        newNode.parent = cur
                        cur.lChild = newNode
                        if (cur.rChild == null) {
                            cur.height++
                            var p = cur.parent
                            while (p != null) {
                                if (!p.updateHeight()) break
                                p = p.parent
                            }
                        }
                        // update head if needed
                        if (head!!.key > key) head = newNode
                        return null
                    } else cur = cur.lChild
                }
                cmp > 0 -> {
                    cur.right++
                    if (cur.rChild == null) {
                        newNode.parent = cur
                        cur.rChild = newNode
                        if (cur.lChild == null) {
                            cur.height++
                            var p = cur.parent
                            while (p != null) {
                                if (!p.updateHeight()) break
                                p = p.parent
                            }
                        }
                        // update tail if needed
                        if (tail!!.key <= key) tail = newNode
                        return null
                    } else cur = cur.rChild
                }
                else -> {
                    // replace value
                    val old = cur.value
                    cur.value = value
                    return old
                }
            }
        }
        return null
    }

    override fun putAll(from: Map<out K, V>) {
        for ((k, v) in from) put(k, v)
    }

    override fun remove(key: K): V? {
        val n = root?.findKey(key) ?: return null
        val old = n.value
        removeNode(n)
        return old
    }

    override fun clear() {
        root = null
        head = null
        tail = null
    }

    // ---------- rank/select helpers ----------

    fun rank(key: K): Int = root?.findKey(key)?.rank() ?: -1

    fun selectEntry(index: Int): Pair<K, V> {
        if (root == null) throw NoSuchElementException()
        if (index < 0 || index >= size) throw NoSuchElementException()
        val n = root!!.select(index)
        return Pair(n.key, n.value)
    }

    // ---------- internal remove logic (mirrors previous implementation) ----------

    private fun removeNode(n: Node?): Boolean {
        if (n == null) return false

        if (n === head) head = head?.next()
        if (n === tail) tail = tail?.prev()

        if (n.lChild == null) {
            var tmp: Node? = n
            while (tmp?.parent != null) {
                if (tmp.parent!!.lChild == tmp) tmp.parent!!.left-- else tmp.parent!!.right--
                tmp = tmp.parent
            }

            if (n.rChild != null) n.rChild!!.parent = n.parent

            if (n.parent == null) {
                root = n.rChild
            } else {
                if (n.parent!!.lChild == n) n.parent!!.lChild = n.rChild else n.parent!!.rChild = n.rChild
            }
            var p = n.parent
            while (p != null) {
                if (!p.updateHeight()) break
                p = p.parent
            }
        } else if (n.rChild == null) {
            var tmp: Node? = n
            while (tmp?.parent != null) {
                if (tmp.parent!!.lChild == tmp) tmp.parent!!.left-- else tmp.parent!!.right--
                tmp = tmp.parent
            }

            n.lChild!!.parent = n.parent

            if (n.parent == null) {
                root = n.lChild
            } else {
                if (n.parent!!.lChild == n) n.parent!!.lChild = n.lChild else n.parent!!.rChild = n.lChild
            }
            var p = n.parent
            while (p != null) {
                if (!p.updateHeight()) break
                p = p.parent
            }
        } else {
            var p = n.prev()
            check(p != null)
            check((p.parent == n && n.lChild == p) || p.parent!!.rChild == p)
            check(p.rChild == null)

            var wasLChild = false
            if (p.parent == n) {
                wasLChild = true
                n.lChild = p.lChild
                if (p.lChild != null) p.lChild!!.parent = n
            } else {
                p.parent!!.rChild = p.lChild
                if (p.lChild != null) p.lChild!!.parent = p.parent
            }
            var tmp: Node? = p
            while (tmp?.parent != null) {
                if ((wasLChild && tmp == p) || tmp.parent!!.lChild == tmp) tmp.parent!!.left-- else tmp.parent!!.right--
                tmp = tmp.parent
            }
            tmp = p
            while (tmp?.parent != null) {
                if (!tmp.parent!!.updateHeight()) break
                tmp = tmp.parent
            }

            p.parent = n.parent
            p.rChild = n.rChild
            p.lChild = n.lChild
            p.left = n.left
            p.right = n.right
            p.height = n.height
            if (p.rChild != null) p.rChild!!.parent = p
            if (p.lChild != null) p.lChild!!.parent = p
            if (n.parent == null) {
                root = p
            } else {
                if (n.parent!!.lChild == n) n.parent!!.lChild = p else n.parent!!.rChild = p
            }
        }
        assert(root == null || root!!.verify())
        return true
    }

    // ---------- views: entries / keys / values ----------

    private inner class MapEntry(val node: Node) : MutableMap.MutableEntry<K, V> {
        override val key: K get() = node.key
        override val value: V get() = node.value
        override fun setValue(newValue: V): V {
            val old = node.value
            node.value = newValue
            return old
        }
    }

    private inner class EntriesView : MutableSet<MutableMap.MutableEntry<K, V>> {
        override val size: Int get() = this@BalancedOrderStatisticTreeMap.size

        override fun isEmpty(): Boolean = this@BalancedOrderStatisticTreeMap.isEmpty()

        override fun contains(element: MutableMap.MutableEntry<K, V>): Boolean {
            val v = get(element.key)
            return v != null && v == element.value
        }

        override fun iterator(): MutableIterator<MutableMap.MutableEntry<K, V>> {
            return object : MutableIterator<MutableMap.MutableEntry<K, V>> {
                private var curr: Node? = head
                private var last: Node? = null

                override fun hasNext(): Boolean = curr != null

                override fun next(): MutableMap.MutableEntry<K, V> {
                    last = curr
                    curr = curr?.next()
                    return MapEntry(last!!)
                }

                override fun remove() {
                    removeNode(last)
                }
            }
        }

        override fun add(element: MutableMap.MutableEntry<K, V>): Boolean {
            put(element.key, element.value)
            return true
        }

        override fun remove(element: MutableMap.MutableEntry<K, V>): Boolean {
            val n = root?.findKey(element.key) ?: return false
            if (n.value != element.value) return false
            removeNode(n)
            return true
        }

        override fun containsAll(elements: Collection<MutableMap.MutableEntry<K, V>>): Boolean = elements.all { contains(it) }
        override fun addAll(elements: Collection<MutableMap.MutableEntry<K, V>>): Boolean { elements.forEach { add(it) }; return true }
        override fun removeAll(elements: Collection<MutableMap.MutableEntry<K, V>>): Boolean { var changed=false; elements.forEach { if (remove(it)) changed=true }; return changed }
        override fun retainAll(elements: Collection<MutableMap.MutableEntry<K, V>>): Boolean {
            val toKeep = elements.map { it.key }.toSet()
            var changed = false
            val it = iterator()
            while (it.hasNext()) {
                val e = it.next()
                if (e.key !in toKeep) { it.remove(); changed = true }
            }
            return changed
        }

        override fun clear() { this@BalancedOrderStatisticTreeMap.clear() }
    }

    override val entries: MutableSet<MutableMap.MutableEntry<K, V>> by lazy { EntriesView() }

    override val keys: MutableSet<K> = object : MutableSet<K> {
        override val size: Int get() = this@BalancedOrderStatisticTreeMap.size
        override fun isEmpty(): Boolean = this@BalancedOrderStatisticTreeMap.isEmpty()
        override fun contains(element: K): Boolean = containsKey(element)
        override fun iterator(): MutableIterator<K> {
            val it = entries.iterator()
            return object : MutableIterator<K> {
                override fun hasNext() = it.hasNext()
                override fun next() = it.next().key
                override fun remove() = it.remove()
            }
        }
        override fun add(element: K): Boolean = throw UnsupportedOperationException()
        override fun remove(element: K): Boolean { val existed = containsKey(element); if (existed) remove(element); return existed }
        override fun containsAll(elements: Collection<K>) = elements.all { contains(it) }
        override fun addAll(elements: Collection<K>): Boolean = throw UnsupportedOperationException()
        override fun retainAll(elements: Collection<K>): Boolean { val toKeep = elements.toSet(); var changed=false; val it = iterator(); while (it.hasNext()) { val k = it.next(); if (k !in toKeep) { it.remove(); changed = true } }; return changed }
        override fun removeAll(elements: Collection<K>): Boolean { var changed=false; elements.forEach { if (remove(it)) changed=true }; return changed }
        override fun clear() = this@BalancedOrderStatisticTreeMap.clear()
    }

    override val values: MutableCollection<V> = object : MutableCollection<V> {
        override val size: Int get() = this@BalancedOrderStatisticTreeMap.size
        override fun isEmpty(): Boolean = this@BalancedOrderStatisticTreeMap.isEmpty()
        override fun contains(element: V): Boolean = containsValue(element)
        override fun iterator(): MutableIterator<V> {
            val it = entries.iterator()
            return object : MutableIterator<V> {
                override fun hasNext() = it.hasNext()
                override fun next() = it.next().value
                override fun remove() = it.remove()
            }
        }
        override fun add(element: V): Boolean = throw UnsupportedOperationException()
        override fun remove(element: V): Boolean {
            val it = iterator()
            while (it.hasNext()) {
                if (it.next() == element) { it.remove(); return true }
            }
            return false
        }
        override fun containsAll(elements: Collection<V>): Boolean = elements.all { contains(it) }
        override fun addAll(elements: Collection<V>): Boolean = throw UnsupportedOperationException()
        override fun removeAll(elements: Collection<V>): Boolean { var changed=false; elements.forEach { if (remove(it)) changed=true }; return changed }
        override fun retainAll(elements: Collection<V>): Boolean { val toKeep = elements.toSet(); var changed=false; val it = iterator(); while (it.hasNext()) { val v = it.next(); if (v !in toKeep) { it.remove(); changed = true } }; return changed }
        override fun clear() = this@BalancedOrderStatisticTreeMap.clear()
    }

    // ---------- in-order traversal helpers ----------

    /**
     * Call [action] for each (key, value) in ascending key order.
     * Iterative, uses the linked `head`/`next()` traversal so it avoids recursion.
     */
    fun forEachInOrder(action: (K, V) -> Unit) {
        var n = head
        while (n != null) {
            action(n.key, n.value)
            n = n.next()
        }
    }

    /**
     * Print the map entries in key order using the provided printer (defaults to println).
     * Format: "key -> value"
     */
    fun printInOrder(printer: (String) -> Unit = ::println) {
        forEachInOrder { k, v -> printer("$k -> $v") }
    }

    /**
     * Provide a string representation of the map in key order. Example: `{a=1, b=2, c=3}`.
     * This is used automatically by `println(tree)`.
     */

    /**
     * Efficiently select the entry at [index] and the next entry (index+1) in a single traversal.
     * Returns a Pair where the first component is the entry at [index] (or null if out-of-bounds),
     * and the second component is the next entry (or null if there is no next).
     *
     * This avoids doing two full `select` traversals: it uses a single select to locate the
     * requested node and then calls `next()` to obtain the immediate successor in O(1) average
     * additional work.
     */
    fun selectPair(index: Int): Pair<Pair<K, V>?, Pair<K, V>?> {
        if (root == null) return Pair(null, null)
        if (index < 0 || index >= size) return Pair(null, null)
        // locate node at index (O(log n))
        val n = root!!.select(index)
        val first = Pair(n.key, n.value)
        val nextNode = n.next()
        val second = nextNode?.let { Pair(it.key, it.value) }
        return Pair(first, second)
    }

    override fun toString(): String {
        val sb = StringBuilder()
        sb.append("{")
        var first = true
        forEachInOrder { k, v ->
            if (!first) sb.append(", ")
            first = false
            sb.append(k).append("=").append(v)
        }
        sb.append("}")
        return sb.toString()
    }
}
