public class Pos(val ident: UInt, val site: UShort) : Comparable<Pos> {

    override fun compareTo(other: Pos): Int {
        val identCmp = ident.compareTo(other.ident)
        if (identCmp != 0) return identCmp
        return site.compareTo(other.site)
    }
}