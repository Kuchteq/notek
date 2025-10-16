package org.example

import Pid
import kotlinx.io.Buffer
import kotlinx.io.
import kotlinx.io.readULongLe
import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.io.IOException
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@OptIn(ExperimentalUuidApi::class)
sealed class SyncRequests {
    data class SyncList(val lastSyncTime: Long) : SyncRequests()
    data class SyncDoc(val lastSyncTime: Long, val documentId: Uuid) : SyncRequests()

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



        companion object {
            fun deserialize(input: Input): DocOp {
                val tag = input.readByte().toInt() and 0xFF

                return when (tag) {
                    0 -> { // Insert
                        val len = input.readByte().toInt() and 0xFF
                        val bytes = input.readByteArray(len)
                        val ch = bytes.toString(Charsets.UTF_8)[0]

                        val depth = input.readByte().toInt() and 0xFF
                        val pid = Pid.fromReader(input, depth)
                        Insert(pid, ch)
                    }

                    1 -> { // Delete
                        val depth = input.readByte().toInt() and 0xFF
                        val pid = Pid.fromReader(input, depth)
                        Delete(pid)
                    }

                    else -> throw IOException("Unknown DocOp tag: $tag")
                }
            }
        }


        fun serializeInto(buf: ByteArrayOutputStream) {
            when (this) {
                is DocOp.Insert -> {
                    buf.write(0)
                    val bytes = ch.toString().toByteArray(Charsets.UTF_8)
                    buf.write(bytes.size)
                    buf.write(bytes)
                    pid.writeBytes(buf)
                }

                is DocOp.Delete -> {
                    buf.write(1)
                    buf.write(pid.depth())
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
                val input = Buffer().apply { write(buf) }

                return when (input.readByte().toInt() and 0xFF) {
                    2 -> { // SyncList
                        val count = input.readULongLe().toInt()
                        val docs = buildList {
                            repeat(count) {
                                val lastModTime = input.readLong()
                                val low = input.readLong().toULong()
                                val high = input.readLong().toULong()
                                val documentId = low or (high shl 64)
                                add(DocSyncInfo(lastModTime, documentId))
                            }
                        }
                        SyncList(docs)
                    }

                    3 -> { // SyncDoc
                        val low = input.readULongLe()
                        val high = input.readULongLe()
                        val documentId = low or (high shl 64)

                        input.readByte() // skip placeholder
                        val updateCount = input.readLong().toInt()

                        val updates = buildList {
                            repeat(updateCount) {
                                add(DocOp.deserialize(input))
                            }
                        }

                        SyncDoc(documentId, updates)
                    }

                    else -> error("Unknown SyncResponses tag")
                }
            }
        }
