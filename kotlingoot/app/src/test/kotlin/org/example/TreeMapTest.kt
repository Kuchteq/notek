package org.example

import dev.kuchta.notek.sync.BalancedOrderStatisticTreeMap
import kotlin.random.Random
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.RepeatedTest
import org.junit.jupiter.api.BeforeEach
import java.util.TreeMap

class TreeMapTests {

    private lateinit var map: BalancedOrderStatisticTreeMap<Int, String>

    @BeforeEach
    fun setup() {
        map = BalancedOrderStatisticTreeMap()
    }

    @Test
    fun `inserts maintain sorted order`() {
        val keys = listOf(5, 2, 8, 1, 9)
        for (k in keys) map[k] = "v$k"

        val sortedKeys = map.keys.toList()
        assertEquals(keys.sorted(), sortedKeys)
    }

    @Test
    fun `get returns correct values`() {
        map[1] = "one"
        map[2] = "two"
        map[3] = "three"

        assertEquals("one", map[1])
        assertEquals("two", map[2])
        assertEquals("three", map[3])
        assertNull(map[4])
    }

    @Test
    fun `removals delete entries correctly`() {
        (1..10).forEach { map[it] = "v$it" }

        map.remove(5)
        map.remove(10)

        assertFalse(map.containsKey(5))
        assertFalse(map.containsKey(10))
        assertEquals(8, map.size)
    }

    @Test
    fun `clear empties the map`() {
        repeat(100) { map[it] = "v$it" }
        map.clear()
        assertTrue(map.isEmpty())
    }

    @RepeatedTest(3)
    fun `massive random insertion and removal remains consistent`() {
        val ref = sortedMapOf<Int, String>() // Reference implementation

        val random = Random(42)
        val keys = (1..10_000).map { random.nextInt(0, 100_000) }

        // Insert phase
        for (k in keys) {
            val value = "v$k"
            map[k] = value
            ref[k] = value
        }

        assertEquals(ref.size, map.size)
        assertEquals(ref.keys.toList(), map.keys.toList())
        assertEquals(ref.values.toList(), map.values.toList())

        // Random removals
        val removeKeys = keys.shuffled(random).take(5000)
        for (k in removeKeys) {
            map.remove(k)
            ref.remove(k)
        }

        assertEquals(ref.size, map.size)
        assertEquals(ref.keys.toList(), map.keys.toList())
    }

    @Test
    fun `iterator traverses in ascending key order`() {
        (1..100).shuffled().forEach { map[it] = "v$it" }

        val iteratedKeys = mutableListOf<Int>()
        for ((k, _) in map) iteratedKeys += k

        assertEquals((1..100).toList(), iteratedKeys)
    }

    @Test
    fun `repeated put overwrites value without growing`() {
        map[1] = "one"
        map[1] = "uno"
        assertEquals(1, map.size)
        assertEquals("uno", map[1])
    }

    @Test
    fun `boundary conditions - empty and single element`() {
        assertTrue(map.isEmpty())

        map[42] = "answer"
        assertEquals(1, map.size)
        assertEquals("answer", map[42])

        map.remove(42)
        assertTrue(map.isEmpty())
    }
}
