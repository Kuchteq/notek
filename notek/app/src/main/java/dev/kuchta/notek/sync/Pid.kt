import kotlinx.io.Source
import kotlinx.io.readUByte
import kotlinx.io.readUIntLe
import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

class Pid(val positions: MutableList<Pos>) : Comparable<Pid> {
    companion object {
        fun new(ident: UInt) = Pid(mutableListOf(Pos(ident, 1u)))

        fun empty() = Pid(mutableListOf())
        fun fromReader(reader: DataInputStream, depth: Int): Pid {
            val positions = mutableListOf<Pos>()

            repeat(depth) {
                // Read 4 bytes (little-endian) for ident
                val identBytes = ByteArray(4)
                reader.readFully(identBytes)
                val ident = ByteBuffer.wrap(identBytes)
                    .order(ByteOrder.LITTLE_ENDIAN)
                    .int
                    .toUInt()

                // Read 1 byte for site
                val site = reader.readUnsignedByte().toUByte()

                positions.add(Pos(ident, site))
            }

            return Pid(positions)
        }

        fun fromSource(source: Source, depth: Int): Pid {
            val positions = mutableListOf<Pos>()

            repeat(depth) {
                val ident = source.readUIntLe()
                val site = source.readUByte()
                positions.add(Pos(ident, site))
            }

            return Pid(positions)
        }
    }
    fun depth() : Int{
        return positions.size
    }
    fun push(p: Pos) {
        this.positions.add(p);
    }
    fun writeBytes(buf: ByteArrayOutputStream) {
        for ( p in positions ) {
            p.writeBytes(buf)
        }
    }
    override fun compareTo(other: Pid): Int {
        for ((a, b) in positions.zip(other.positions)) {
            val cmp = a.compareTo(b)
            if (cmp != 0) return cmp
        }
        return positions.size.compareTo(other.positions.size)
    }
}
