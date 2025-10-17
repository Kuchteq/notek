package org.example

import Pid
import kotlinx.io.*
import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.io.IOException
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@OptIn(ExperimentalUuidApi::class)
sealed class SyncRequests {
    data class SyncList(val lastSyncTime: ULong) : SyncRequests()
    data class SyncDoc(val lastSyncTime: ULong, val documentId: Uuid) : SyncRequests()

}

@OptIn(ExperimentalUuidApi::class)
data class DocSyncInfo(val lastModTime: ULong, val documentId: Uuid)

sealed class DocOp {
    data class Insert(val pid: Pid, val ch: Char) : DocOp()
    data class Delete(val pid: Pid) : DocOp()



        companion object {
            fun deserialize(source: Source) : DocOp {
                when (source.readUByte().toUInt()) {
                    0u -> {
                        val data_len = source.readUByte().toInt();
                        val data = source.readByteArray(data_len)
                        val char = data.toString(Charsets.UTF_8)[0]
                        val pid_depth = source.readUByte()
                        Pid.fromSource(source, pid_depth.toInt())
                    }
                    1u -> {

                    }
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

@OptIn(ExperimentalUuidApi::class)
    sealed class SyncResponses {
        data class SyncList(val docs: List<DocSyncInfo>) : SyncResponses()
        data class SyncDoc(val documentId: ULong, val updates: List<DocOp>) : SyncResponses()
        companion object {
            fun deserialize(source: Source) : SyncResponses {
                val header = source.readUByte().toUInt()
                when (header) {
                    0u -> {
                        val numberOfDocuments = source.readULongLe();
                        val docs = mutableListOf<DocSyncInfo>()
                        repeat (numberOfDocuments.toInt()) {
                            val lastModified = source.readULongLe()
                            val docIdBa = source.readByteArray(16)
                            val docId = Uuid.fromByteArray(docIdBa)
                            docs.add(DocSyncInfo(lastModified, docId))
                        }
                        return SyncList(docs)
                    }
                    1u -> {
                        val docIdBa = source.readByteArray(16)
                        val docId = Uuid.fromByteArray(docIdBa)
                        val syncStyle = source.readUByte()
                        val numberOfAtoms = source.readULongLe()
                        repeat(numberOfAtoms.toInt()) {

                        }

                    }
                }
            }
        }
    }
