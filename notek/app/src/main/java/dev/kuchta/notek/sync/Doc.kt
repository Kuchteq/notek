import kotlinx.io.*
import java.io.DataInputStream
import java.util.TreeMap
import kotlin.math.max
import kotlin.times
import java.io.ByteArrayOutputStream
import java.nio.charset.StandardCharsets

data class Doc(
    val content: TreeMap<Pid, Char>,
    val site: UByte
) {
    companion object {
        fun fromSource(source: Source, n: Int): Doc {
            val content = TreeMap<Pid, Char>() // same semantics as TreeMap

            repeat(n) {
                // Read the UTF-8 character length (1–4 bytes)
                val dataLen = source.readUByte().toInt()

                // Read up to 4 bytes of UTF-8 data
                val data = source.readByteArray(dataLen)
                val char = data.toString(Charsets.UTF_8)[0]

                // Read pid depth and pid itself
                val pidDepth = source.readUByte().toInt()
                val pid = Pid.fromSource(source, pidDepth)

                // Insert into map
                content[pid] = char
            }

            // Return Doc with site hard-coded as 1
            return Doc(content, 1u)
        }

        fun fromReader(reader: DataInputStream, n: Int): Doc {
            val content = TreeMap<Pid, Char>() // same semantics as BTreeMap

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
            return Doc(content, 1u)
        }
        fun fromString(content: String, site: UByte) : Doc {
            val step = UInt.MAX_VALUE / content.length.toUInt();
            var map = TreeMap<Pid, Char>();
            val beg = Pid.new(0u)
            val end = Pid.new(UInt.MAX_VALUE)

            map[beg] = '_'
            map[end] = '_'
            for ((i, c) in content.withIndex()) {
                val v = Pid.new(i.toUInt()*step)
                map[v] = c
            }
            return Doc(map, site)
        }
    }
    fun writeBytes(buf: ByteArrayOutputStream) {
        for ((pid, ch) in content) {
            // 1️⃣ Encode the character into UTF-8 bytes (1–4 bytes)
            val encoded = ch.toString().toByteArray(StandardCharsets.UTF_8)

            // 2️⃣ Write the encoded character length (u8)
            buf.write(encoded.size)

            // 3️⃣ Write the actual UTF-8 bytes
            buf.write(encoded)

            // 4️⃣ Write pid depth (u8)
            buf.write(pid.depth().toInt())

            // 5️⃣ Write the pid vector itself
            pid.writeBytes(buf)
        }
    }
    fun display() : String {
        return this.content.values.joinToString(separator = "")
    }

    public fun delete(pid: Pid) {
        this.content.remove(pid)
    }
    public fun insert(pid: Pid, c: Char) {
        this.content[pid] = c
    }
    public fun insert_left(pid: Pid, c: Char, siteId: UByte) {
        val new = generate_between_pids(pid, right(pid), siteId)
        this.content[new] = c;
    }

    public fun right(pid: Pid) : Pid {
        return this.content.higherKey(pid)
    }

    public fun left(pid: Pid) : Pid {
       return this.content.lowerKey(pid)
    }

    public fun len() : Int {
        return this.content.size
    }

}

public fun generate_between_pids(lp: Pid, rp: Pid, site_id: UByte) : Pid {
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