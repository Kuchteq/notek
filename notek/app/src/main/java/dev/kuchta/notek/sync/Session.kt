package org.example
import Pid
import dev.kuchta.notek.sync.DocOp
import kotlinx.io.*
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.UUID
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.toKotlinUuid

@OptIn(ExperimentalUuidApi::class)
sealed class Session {
    abstract fun serialize(sink: Sink)

    data class Start(val lastSyncTime: ULong, val documentId: UUID) : Session() {
        override fun serialize(sink: Sink) {
            sink.writeUByte(64u)
            sink.writeLongLe(lastSyncTime.toLong())
            sink.write(documentId.toKotlinUuid().toByteArray())
        }
    }

    data class Insert(val site: UByte, val pid: Pid, val c: Char) : Session() {
        override fun serialize(sink: Sink) {
            sink.writeUByte(65u)
            sink.writeUByte(site)
            DocOp.Insert(pid, c).serialize(sink)
        }
    }

    data class Delete(val site: UByte, val pid: Pid) : Session() {
        override fun serialize(sink: Sink) {
            sink.writeUByte(66u)
            sink.writeUByte(site)
            DocOp.Delete(pid).serialize(sink)
        }
    }

    companion object {
        fun deserialize(source: Source): Session {
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
                    val op = DocOp.Insert.deserialize(source)
                    Insert(site, op.pid, op.ch)
                }
                66u -> {
                    val site = source.readUByte()
                    val op = DocOp.Delete.deserialize(source)
                    Delete(site, op.pid)
                }
                else -> throw IllegalArgumentException("Unknown message tag: $tag")
            }
        }
    }
    fun serialized(): ByteArray {
        val bytes = Buffer();
        this.serialize(bytes)
        return bytes.readByteArray()
    }
}
