package org.example
import Doc
import Pid
import kotlinx.io.*
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.UUID
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid
import kotlin.uuid.putUuid

@OptIn(ExperimentalUuidApi::class)
sealed class PeerMessage {
    abstract fun serialize(sink: Sink)

    data class Start(val lastSyncTime: ULong, val documentId: UUID) : PeerMessage() {
        override fun serialize(sink: Sink) {
            sink.writeUByte(64u)
            sink.writeLongLe(lastSyncTime.toLong())
            val bb = ByteBuffer.allocate(16).order(ByteOrder.LITTLE_ENDIAN)
            bb.putLong(documentId.mostSignificantBits)
            bb.putLong(documentId.leastSignificantBits)
            sink.write(bb.array())
        }
    }

    data class Insert(val site: UByte, val pid: Pid, val c: Char) : PeerMessage() {
        override fun serialize(sink: Sink) {
            sink.writeUByte(66u)
            sink.writeUByte(site)
            val encoded = c.toString().toByteArray(Charsets.UTF_8)
            sink.writeUByte(encoded.size.toUByte())
            sink.write(encoded)
            sink.writeUByte(pid.depth().toUByte())
            pid.writeTo(sink)
        }
    }

    data class Delete(val site: UByte, val pid: Pid) : PeerMessage() {
        override fun serialize(sink: Sink) {
            sink.writeUByte(67u)
            sink.writeUByte(site)
            sink.writeUByte(pid.depth().toUByte())
            pid.writeTo(sink)
        }
    }

    companion object {
        fun deserialize(source: Source): PeerMessage {
            return when (val tag = source.readUByte().toUInt()) {
                64u -> {
                    val lastSyncTime = source.readLongLe().toULong()
                    val uuidBytes = source.readByteArray(16)
                    val bb = ByteBuffer.wrap(uuidBytes).order(ByteOrder.LITTLE_ENDIAN)
                    val uuid = UUID(bb.long, bb.long)
                    Start(lastSyncTime, uuid)
                }
                65u -> {
                    val site = source.readUByte()
                    val dataLen = source.readUByte().toInt()
                    val data = source.readByteArray(dataLen)
                    val c = data.toString(Charsets.UTF_8).first()
                    val pidDepth = source.readUByte().toInt()
                    val pid = Pid.fromSource(source, pidDepth)
                    Insert(site, pid, c)
                }
                66u -> {
                    val site = source.readUByte()
                    val pidDepth = source.readUByte().toInt()
                    val pid = Pid.fromSource(source, pidDepth)
                    Delete(site, pid)
                }
                else -> throw IllegalArgumentException("Unknown message tag: $tag")
            }
        }
    }
}
