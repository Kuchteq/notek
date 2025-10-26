package dev.kuchta.notek.sync

import Pid
import kotlinx.io.Sink
import kotlinx.io.Source
import kotlinx.io.readByteArray
import kotlinx.io.readUByte
import kotlinx.io.writeUByte

sealed class DocOp {

    data class Insert(val pid: Pid, val ch: Char) : DocOp() {
        companion object {
            fun deserialize(source: Source): Insert {
                val dataLen = source.readUByte().toInt()
                val data = source.readByteArray(dataLen)
                val char = data.toString(Charsets.UTF_8)[0]
                val pidDepth = source.readUByte()
                val pid = Pid.fromSource(source, pidDepth.toInt())
                return Insert(pid, char)
            }
        }

        override fun serialize(sink: Sink) {
            val bytes = ch.toString().toByteArray(Charsets.UTF_8)
            sink.writeUByte(bytes.size.toUByte())
            sink.write(bytes)
            sink.writeUByte(pid.depth().toUByte())
            pid.writeTo(sink)
        }
    }

    data class Delete(val pid: Pid) : DocOp() {
        companion object {
            fun deserialize(source: Source): Delete {
                val pidDepth = source.readUByte()
                val pid = Pid.fromSource(source, pidDepth.toInt())
                return Delete(pid)
            }
        }

        override fun serialize(sink: Sink) {
            sink.writeUByte(pid.depth().toUByte())
            pid.writeTo(sink)
        }
    }

//    companion object {
//        fun deserialize(source: Source): DocOp {
//            return when (source.readUByte().toUInt()) {
//                0u -> Insert.deserialize(source)
//                1u -> Delete.deserialize(source)
//                else -> throw Exception("Bad Op")
//            }
//        }
//    }

    abstract fun serialize(sink: Sink)
}
