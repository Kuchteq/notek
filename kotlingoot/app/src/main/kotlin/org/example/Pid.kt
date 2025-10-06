class Pid(val positions: MutableList<Pos>) : Comparable<Pid> {
    companion object {
        fun new(ident: UInt) = Pid(mutableListOf(Pos(ident, 1u)))

        fun empty() = Pid(mutableListOf())

    }

    fun push(p: Pos) {
        this.positions.add(p);
    }
    override fun compareTo(other: Pid): Int {
        for ((a, b) in positions.zip(other.positions)) {
            val cmp = a.compareTo(b)
            if (cmp != 0) return cmp
        }
        return positions.size.compareTo(other.positions.size)
    }
}
