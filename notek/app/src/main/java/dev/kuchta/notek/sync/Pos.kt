import kotlinx.io.Sink
import kotlinx.io.writeIntLe
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

public class Pos(val ident: UInt, val site: UByte) : Comparable<Pos> {

    override fun compareTo(other: Pos): Int {
        val identCmp = ident.compareTo(other.ident)
        if (identCmp != 0) return identCmp
        return site.compareTo(other.site)
    }
    fun writeBytes(buf: ByteArrayOutputStream) {
        val identBytes = ByteBuffer.allocate(4)
            .order(ByteOrder.LITTLE_ENDIAN)
            .putInt(ident.toInt())
            .array()
        buf.write(identBytes)

        buf.write(byteArrayOf(site.toByte()))
    }
    fun writeTo(sink: Sink) {
        // Write 4-byte unsigned int in little-endian order
        sink.writeIntLe(ident.toInt())

        // Write 1-byte unsigned site ID
        sink.writeByte(site.toByte())
    }

    override fun toString(): String = "($ident@$site)"
}