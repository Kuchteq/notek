import dev.kuchta.notek.sync.BalancedOrderStatisticTreeMap
import dev.kuchta.notek.sync.DocOp
import kotlinx.io.*
import java.io.DataInputStream
import java.util.TreeMap
import kotlin.math.max
import java.nio.charset.StandardCharsets

data class Doc(
    val content: BalancedOrderStatisticTreeMap<Pid, Char>,
) {
    companion object {
        fun fromSource(source: Source, n: Int): Doc {
            val content = BalancedOrderStatisticTreeMap<Pid, Char>() // same semantics as TreeMap

            repeat(n) {
                val dataLen = source.readUByte().toInt()
                val data = source.readByteArray(dataLen)
                val char = data.toString(Charsets.UTF_8)[0]
                val pidDepth = source.readUByte().toInt()
                val pid = Pid.fromSource(source, pidDepth)
                content[pid] = char
            }

            return Doc(content)
        }

        fun fromSource(source: Source): Doc {
            val content = BalancedOrderStatisticTreeMap<Pid, Char>()

            while (!source.exhausted()) {
                val dataLen = source.readUByte().toInt()
                val data = source.readByteArray(dataLen)
                val char = data.toString(Charsets.UTF_8)[0]
                val pidDepth = source.readUByte().toInt()
                val pid = Pid.fromSource(source, pidDepth)
                content[pid] = char
            }
            return Doc(content)
        }
        fun empty(): Doc {
            var map = BalancedOrderStatisticTreeMap<Pid, Char>();
            val beg = Pid.new1d(0u,  0u)
            val end = Pid.new1d(UInt.MAX_VALUE, 0u)

            map[beg] = '_'
            map[end] = '_'
            return Doc(map)
        }
        fun fromInsertsAndDeletes(inserts: List<DocOp.Insert>, deletes: List<DocOp.Delete>) : Doc {
            val d = empty()
            for (insert in inserts) {
                d.insert(insert.pid, insert.ch)
            }
            return d
        }
        fun fromReader(reader: DataInputStream, n: Int): Doc {
            val content = BalancedOrderStatisticTreeMap<Pid, Char>() // same semantics as BTreeMap

            repeat(n) {
                // Read the character length (1–4 bytes for UTF-8)
                val dataLen = reader.readUnsignedByte()

                // Read up to 4 bytes of UTF-8 data
                val bytes = ByteArray(4)
                reader.readFully(bytes, 0, dataLen)

                // Decode only the first character
                val data = bytes.decodeToString(0, dataLen).first()

                // Read the pid depth and the pid itself
                val pidDepth = reader.readUnsignedByte()
                val pid = Pid.fromReader(reader, pidDepth)

                // Insert into the map
                content[pid] = data
            }

            // Return a new Doc (site hard-coded as 1)
            return Doc(content)
        }
        fun fromString(content: String) : Doc {
            val d = empty()
            val step = UInt.MAX_VALUE / content.length.toUInt();
            for ((i, c) in content.withIndex()) {
                val v = Pid.new1d(i.toUInt()*step, 0u)
                d.insert(v, c)
            }
            return d
        }
    }
    fun insertAtPhysicalOrder(p: Int, ch: Char): Pid? {
        val (left, right) = this.content.selectPair(p)
        if (left != null && right != null) {
            val pid = generate_between_pids(left.first,right.first, 0u)
            insert(pid, ch)
            return pid
        }
        return null
    }
    fun deleteAtPhysicalOrder(p: Int): Pid {
        println("deleting $p")
        val (pid, _) = this.content.selectEntry(p)
        delete(pid)
        return pid
    }

    fun writeTo(sink: Sink) {
        for ((pid, ch) in content) {
            val encoded = ch.toString().toByteArray(StandardCharsets.UTF_8)
            sink.writeByte(encoded.size.toByte())
            sink.write(encoded)
            sink.writeByte(pid.depth().toByte())
            pid.writeTo(sink)
        }
    }
    fun serialized(): ByteArray {
        val bytes = Buffer();
        this.writeTo(bytes)
        return bytes.readByteArray()
    }

    fun display() : String {
        return this.content.values.drop(1).dropLast(1).joinToString(separator = "")
    }

    public fun delete(pid: Pid) {
        this.content.remove(pid)
    }
    public fun insert(pid: Pid, c: Char) {
        this.content[pid] = c
    }
//    public fun insert_left(pid: Pid, c: Char, siteId: UByte) {
//        val new = generate_between_pids(pid, right(pid), siteId)
//        this.content[new] = c;
//    }
//
//    public fun right(pid: Pid) : Pid {
//        return this.content.higherKey(pid)
//    }
//
//    public fun left(pid: Pid) : Pid {
//       return this.content.lowerKey(pid)
//    }

    public fun len() : Int {
        return this.content.size
    }

}

public fun generate_between_pids(lp: Pid, rp: Pid, site_id: UByte) : Pid {
    val p = Pid.empty()

    val max_depth = max(lp.positions.size, rp.positions.size)
    for (i in 0 until max_depth) {
        val l : Pos = lp.positions.getOrElse(i) { Pos(0u, 0u) }
        val r : Pos = rp.positions.getOrElse(i) { Pos(UInt.MAX_VALUE, 0u) }
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
//            val new_ident = (l.ident + 1u..r.ident).random();
            val new_ident = (l.ident.toULong() + 1u + r.ident.toULong())/2u;
            p.push(Pos(new_ident.toUInt(), site_id))
            return p
        } else {
            p.push(l)
        }
    }
//    p.push(Pos((0u..UInt.MAX_VALUE).random(),site_id))
    p.push(Pos((0u + UInt.MAX_VALUE) / 2u,site_id))
    return p
}