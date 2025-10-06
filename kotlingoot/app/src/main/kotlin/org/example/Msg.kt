package org.example

import Pid
import com.fasterxml.jackson.annotation.JsonSubTypes
import com.fasterxml.jackson.annotation.JsonTypeInfo

@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type")
@JsonSubTypes(
    JsonSubTypes.Type(value = PeerMessage.Greet::class, name = "Greet"),
    JsonSubTypes.Type(value = PeerMessage.Insert::class, name = "Insert"),
    JsonSubTypes.Type(value = PeerMessage.Delete::class, name = "Delete"),
)
sealed class PeerMessage {
    data object Greet : PeerMessage()

    data class Insert(val version: UByte, val pid: Pid, val ch: Char) : PeerMessage()

    data class Delete(val version: UByte, val pid: Pid) : PeerMessage()
}