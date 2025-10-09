package org.example

import Pid
import com.fasterxml.jackson.annotation.JsonSubTypes
import com.fasterxml.jackson.annotation.JsonTypeInfo
import java.util.TreeMap

data class Doc(
    val content: TreeMap<Pid, Char>,
    val site: UByte
)

@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes(
    JsonSubTypes.Type(value = PeerMessage.Greet::class, name = "Greet"),
    JsonSubTypes.Type(value = PeerMessage.Insert::class, name = "Insert"),
    JsonSubTypes.Type(value = PeerMessage.Delete::class, name = "Delete"),
    JsonSubTypes.Type(value = PeerMessage.NewSession::class, name = "NewSession"),
    JsonSubTypes.Type(value = PeerMessage.NewSessionRaw::class, name = "NewSessionRaw"),
    )
sealed class PeerMessage {
    data object Greet : PeerMessage()

    data class Insert(val version: UByte, val pid: Pid, val ch: Char) : PeerMessage()

    data class Delete(val version: UByte, val pid: Pid) : PeerMessage()

    data class NewSession(val version: Int, val doc: Doc) : PeerMessage()

    data class NewSessionRaw(val version: Int, val keys: MutableList<Pid>, val values: MutableList<Char>) : PeerMessage()
}