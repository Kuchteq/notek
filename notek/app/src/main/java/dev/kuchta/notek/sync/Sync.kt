package org.example

import Doc
import Pid
import dev.kuchta.notek.sync.DocOp
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
    fun serialized(): ByteArray {
        val bytes = Buffer();
        this.serialize(bytes)
        return bytes.readByteArray()
    }
    companion object {
    }
}

@OptIn(ExperimentalUuidApi::class)
data class DocSyncInfo(val lastModTime: ULong, val documentId: Uuid)


@OptIn(ExperimentalUuidApi::class)
    sealed class SyncResponses {
        data class SyncList(val docs: List<DocSyncInfo>) : SyncResponses()
        data class SyncDoc(val documentId: Uuid, val name: String, val inserts: List<DocOp.Insert>, val deletes: List<DocOp.Delete>) : SyncResponses()

        companion object {
            fun deserialize(source: Source) : SyncResponses {
                val header = source.readUByte().toUInt()
                when (header) {
                    32u -> {
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
                    33u -> {
                        val docId = Uuid.fromByteArray(source.readByteArray(16))
                        val documentName = source.readLine()!!
                        val numberOfInsertAtoms = source.readULongLe()
                        val inserts = mutableListOf<DocOp.Insert>()
                        repeat(numberOfInsertAtoms.toInt()) {
                            inserts.add(DocOp.Insert.deserialize(source))
                        }

                        val numberOfDeletes = source.readULongLe()
                        val deletes = mutableListOf<DocOp.Delete>()
                        repeat(numberOfDeletes.toInt()) {
                            deletes.add(DocOp.Delete.deserialize(source))
                        }
                        return SyncDoc(docId, documentName, inserts, deletes)

                    }
                    else -> throw Exception("Bad Sync Class")
                }
            }
        }
    }
