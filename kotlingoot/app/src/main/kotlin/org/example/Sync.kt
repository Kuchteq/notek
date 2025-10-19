package org.example

import Doc
import Pid
import kotlinx.io.*
import java.io.ByteArrayOutputStream
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@OptIn(ExperimentalUuidApi::class)
sealed class SyncRequests {
    data class SyncList(val lastSyncTime: ULong) : SyncRequests()
    data class SyncDoc(val lastSyncTime: ULong, val documentId: Uuid) : SyncRequests()

    fun serialize(sink: Sink) {
        when (this) {
            is SyncList -> {
                sink.writeUByte(0u);
                sink.writeULong(0u)
            }
            is SyncDoc -> {
                sink.writeUByte(1u);
                sink.writeULong(0u)
                sink.write(documentId.toByteArray())
            }
        }
    }
    companion object {
    }
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
                        val pid = Pid.fromSource(source, pid_depth.toInt())
                        return Insert(pid, char)
                    }
                    1u -> {
                        val pid_depth = source.readUByte()
                        val pid = Pid.fromSource(source, pid_depth.toInt())
                        return Delete(pid)
                    }
                    else -> throw Exception("Bad Op")
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
        data class SyncOpDoc(val documentId: Uuid, val updates: List<DocOp>) : SyncResponses()

        data class SyncFullDoc(val documentId: Uuid, val doc: Doc) : SyncResponses()

        companion object {
            fun deserialize(source: Source) : SyncResponses {
                val header = source.readUByte().toUInt()
                when (header) {
                    2u -> {
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
                    3u -> {
                        val docIdBa = source.readByteArray(16)
                        val docId = Uuid.fromByteArray(docIdBa)
                        val numberOfAtoms = source.readULongLe()
                        var updates = mutableListOf<DocOp>()
                        repeat(numberOfAtoms.toInt()) {
                            updates.add(DocOp.deserialize(source))
                        }
                        return SyncOpDoc(docId, updates)

                    }
                    4u -> {
                        val docIdBa = source.readByteArray(16)
                        val docId = Uuid.fromByteArray(docIdBa)
                        val numberOfAtoms = source.readULongLe()
                        val doc = Doc.fromSource(source, numberOfAtoms.toInt())
                        return SyncFullDoc(docId, doc)
                    }
                    else -> throw Exception("Bad Sync Class")
                }
            }
        }
    }
