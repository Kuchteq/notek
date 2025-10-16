package org.example

import Pid
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@OptIn(ExperimentalUuidApi::class)
sealed class SyncRequests {
    data class SyncList(val lastSyncTime: Long) : SyncRequests()
    data class SyncDoc (val lastSyncTime: Long, val documentId: Uuid) : SyncRequests()

    fun serialize(): ByteArray {
        val buf = ByteArrayOutputStream()
        when (this) {
            is SyncList -> {
                buf.write(0)
                val lastSyncTimeBytes = ByteBuffer.allocate(8)
                    .order(ByteOrder.LITTLE_ENDIAN)
                    .putLong(lastSyncTime)
                    .array()
                buf.write(lastSyncTimeBytes)
            }
            is SyncDoc -> {
                buf.write(1)
                val lastSyncTimeBytes = ByteBuffer.allocate(8)
                    .order(ByteOrder.LITTLE_ENDIAN)
                    .putLong(lastSyncTime)
                    .array()
                buf.write(lastSyncTimeBytes)
                buf.write(documentId.toByteArray())
            }
        }
        return buf.toByteArray()
    }
}
data class DocSyncInfo(val lastModTime: Long, val documentId: ULong)

sealed class DocOp {
    data class Insert(val pid: Pid, val ch: Char) : DocOp()
    data class Delete(val pid: Pid) : DocOp()

    fun serializeInto(buf: MutableList<Byte>) {
        when (this) {
            is DocOp.Insert -> {
                buf.add(0)
                val bytes = ch.toString().toByteArray(Charsets.UTF_8)
                buf.add(bytes.size.toByte())
                buf.addAll(bytes.toList())
                pid.writeBytes(buf)
            }
            is DocOp.Delete -> {
                buf.add(1)
                buf.add(pid.parts.size.toByte())
                pid.writeBytes(buf)
            }
        }
    }
}

sealed class SyncResponses {
    data class SyncList(val docs: List<DocSyncInfo>) : SyncResponses()
    data class SyncDoc(val documentId: ULong, val updates: List<DocOp>) : SyncResponses()

    companion object {
        fun deserialize(buf: ByteArray): SyncResponses {
            val bb = java.nio.ByteBuffer.wrap(buf).order(java.nio.ByteOrder.LITTLE_ENDIAN)
            return when (bb.get().toInt() and 0xFF) {
                2 -> { // SyncList
                    val count = bb.long.toInt()
                    val docs = mutableListOf<DocSyncInfo>()
                    repeat(count) {
                        val lastModTime = bb.long
                        val documentId = bb.long.toULong() or (bb.long.toULong() shl 64)
                        docs.add(DocSyncInfo(lastModTime, documentId))
                    }
                    SyncList(docs)
                }
                3 -> { // SyncDoc
                    val documentId = bb.long.toULong() or (bb.long.toULong() shl 64)
                    bb.get() // skip placeholder
                    val updateCount = bb.long.toInt()
                    val updates = mutableListOf<DocOp>()
                    repeat(updateCount) {
                        updates.add(DocOp.deserialize(bb))
                    }
                    SyncDoc(documentId, updates)
                }
                else -> throw IllegalArgumentException("Unknown discriminant")
            }
        }
    }
}
