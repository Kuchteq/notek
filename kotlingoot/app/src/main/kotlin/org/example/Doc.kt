import java.util.TreeMap
import kotlin.math.max


public class Doc {
    val content: TreeMap<Pid, Char>
    var site: UShort

    // Primary constructor
    constructor(content: String, site: UShort) {
        val step = UInt.MAX_VALUE / content.length.toUInt();
        this.content = TreeMap();
        val beg = Pid.new(0u)
        val end = Pid.new(UInt.MAX_VALUE)

        this.content[beg] = '_'
        this.content[end] = '_'
        for ((i, c) in content.withIndex()) {
            val v = Pid.new(i.toUInt()*step)
            this.content[v] = c
        }
        this.site = site
    }

    public fun insert_left(pid: Pid, c: Char, siteId: UShort) {
        val new = generate_between_pids(pid, right(pid), siteId)
        this.content[new] = c;
    }

    public fun right(pid: Pid) : Pid {
        return this.content.higherKey(pid)
    }

    public fun left(pid: Pid) : Pid {
       return this.content.lowerKey(pid)
    }

}

public fun generate_between_pids(lp: Pid, rp: Pid, site_id: UShort) : Pid {
    var p = Pid.empty()

    val max_depth = max(lp.positions.size, rp.positions.size)
    for (i in 0 until max_depth) {
        val l : Pos = lp.positions.getOrElse(i) { Pos(0u, 0u) }
        val r : Pos = lp.positions.getOrElse(i) { Pos(UInt.MAX_VALUE, 0u) }

        if (l == r) {
            p.push(l);
            continue;
        }

        if (l.ident == r.ident) {
            p.push(Pos(l.ident, site_id))
            return p;
        }
        val d = r.ident - l.ident
        if ( d > 1u) {
            val new_ident = (l.ident + 1u..r.ident).random();
            p.push(Pos(new_ident, site_id))
            return p
        } else {
            p.push(l)
        }
    }
    p.push(Pos((0u..UInt.MAX_VALUE).random(),site_id))
    return p
}