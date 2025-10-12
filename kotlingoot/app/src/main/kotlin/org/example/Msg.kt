package org.example
import Doc
import Pid
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.io.DataOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

sealed class PeerMessage {
    object Greet : PeerMessage()

    data class Insert(val site: UByte, val pid: Pid, val c: Char) : PeerMessage()
    data class Delete(val site: UByte, val pid: Pid) : PeerMessage()
    data class NewSession(val site: UByte, val doc: Doc) : PeerMessage()

    fun serialize(): ByteArray {
        return when (this) {
            is Greet -> byteArrayOf(0u.toByte())

            is NewSession -> {
                val buf = ByteArrayOutputStream()
                buf.write(1)
                buf.write(site.toInt())
                val lenBytes = ByteBuffer.allocate(8)
                    .order(ByteOrder.LITTLE_ENDIAN)
                    .putLong(doc.len().toLong())
                    .array()
                buf.write(lenBytes)
                doc.writeBytes(buf)
                buf.toByteArray()
            }

            is Insert -> {
                val buf = ByteArrayOutputStream()
                buf.write(2)
                buf.write(site.toInt())
                val encoded = c.toString().toByteArray(Charsets.UTF_8)
                buf.write(encoded.size)
                buf.write(encoded)
                buf.write(pid.depth().toInt())
                pid.writeBytes(buf)
                buf.toByteArray()
            }

            is Delete -> {
                val buf = ByteArrayOutputStream()
                buf.write(3)
                buf.write(site.toInt())
                buf.write(pid.depth().toInt())
                pid.writeBytes(buf)
                buf.toByteArray()
            }
        }
    }

    companion object {
        fun deserialize(buf: ByteArray): PeerMessage {
            val input = DataInputStream(ByteArrayInputStream(buf))
            return when (val tag = input.readUnsignedByte()) {
                0 -> Greet

                1 -> {
                    val site = input.readUnsignedByte().toUByte()
                    val numberOfAtoms = java.lang.Long.reverseBytes(input.readLong()).toInt()
                    val doc = Doc.fromReader(input, numberOfAtoms)
                    NewSession(site, doc)
                }

                2 -> {
                    val site = input.readUnsignedByte().toUByte()
                    val dataLen = input.readUnsignedByte()
                    val bytes = ByteArray(4)
                    input.read(bytes, 0, dataLen)
                    val c = bytes.decodeToString(0, dataLen).first()
                    val pidDepth = input.readUnsignedByte()
                    val pid = Pid.fromReader(input, pidDepth)
                    Insert(site, pid, c)
                }

                3 -> {
                    val site = input.readUnsignedByte().toUByte()
                    val pidDepth = input.readUnsignedByte()
                    val pid = Pid.fromReader(input, pidDepth)
                    Delete(site, pid)
                }

                else -> throw IllegalArgumentException("Unknown message tag: $tag")
            }
        }
    }
}
