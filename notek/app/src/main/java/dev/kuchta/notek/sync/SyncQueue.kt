import androidx.compose.foundation.text.input.TextFieldState
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.kuchta.notek.Note
import dev.kuchta.notek.g
import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.websocket.*
import io.ktor.http.*
import io.ktor.websocket.*
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.io.Buffer
import org.example.SyncRequests
import org.example.SyncResponses
import java.util.UUID
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.toJavaUuid
import kotlin.uuid.toKotlinUuid

sealed class SyncQueueItem {
    data class DeleteNote(val id: UUID): SyncQueueItem()
    object Sync: SyncQueueItem()
}

@OptIn(ExperimentalUuidApi::class)
class SyncQueue : ViewModel() {
    private val outgoingQueue = Channel<SyncQueueItem>(Channel.UNLIMITED)

    fun enqueue(item: SyncQueueItem) {
        viewModelScope.launch {
            outgoingQueue.send(item)
        }
    }

    val serverUrl =
        TextFieldState(initialText = g.sharedPreferences.getString("serverUrl", "").orEmpty())

    val client = HttpClient(CIO) {
        install(WebSockets)
    }

    private val dao = g.db.noteDao()

    private suspend fun DefaultClientWebSocketSession.handleSending() {
            try {
                for (item in outgoingQueue) {
                    when (item) {
                        is SyncQueueItem.DeleteNote -> {
                            val req = SyncRequests.DeleteNote(item.id.toKotlinUuid())
                            dao.deleteNoteById(item.id)
                            send(req.serialized())
                        }
                        is SyncQueueItem.Sync -> {
                            val req = SyncRequests.SyncList(0u)
                            send(req.serialized())
                        }
                    }
                }
            } catch (e: Exception) {
                println("Sender terminated: ${e.message}")
            }
    }

    private suspend fun DefaultClientWebSocketSession.handleReceiving() {
        try {
            for (frame in incoming) {
                val buffer = Buffer().apply { write(frame.readBytes()) }
                val resp = SyncResponses.deserialize(buffer)

                when (resp) {
                    is SyncResponses.SyncDoc -> {
                        val crdt = Doc.fromInsertsAndDeletes(resp.inserts, resp.deletes)
                        println(crdt.content)
                        dao.insert(
                            Note(
                                resp.documentId.toJavaUuid(),
                                name = resp.name,
                                crdt.display(),
                                0,
                                crdt.serialized()
                            )
                        )
                    }
                    is SyncResponses.SyncList -> {
                        for (doc in resp.docs) {
                            val sr = SyncRequests.SyncDoc(0u, doc.documentId)
                            send(sr.serialized())
                        }
                    }
                    else -> {}
                }
            }
        } catch (e: Exception) {
            println("Receiver terminated: ${e.message}")
        }
    }
    @OptIn(ExperimentalUuidApi::class)
    fun startProcessing() {
        viewModelScope.launch {
            while (isActive) {
                try {
                    client.ws(method = HttpMethod.Get, host = serverUrl.text.toString(), port = 9001, path = "/") {
                        println("Connected to server: ${serverUrl.text}")

                        val sender = launch {
                            handleSending()
                        }
                        val receiver = launch {
                            handleReceiving()
                        }
                        joinAll(receiver, sender)
                    }
                } catch (e: Exception) {
                    println("Connection failed: ${e.message}. Retrying in 2 seconds...")
                    delay(2000)
                }
            }
        }
    }

    override fun onCleared() {
        super.onCleared()
        client.close()
    }
}
