package dev.kuchta.notek.setup

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.kuchta.notek.Note
import dev.kuchta.notek.g
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.ws
import io.ktor.http.HttpMethod
import io.ktor.websocket.readBytes
import io.ktor.websocket.send
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.io.Buffer
import kotlinx.io.readByteArray
import org.example.SyncRequests
import org.example.SyncResponses
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.toJavaUuid

@OptIn(FlowPreview::class)
class SetupViewModel : ViewModel() {

    val serverUrl =
        TextFieldState(initialText = g.sharedPreferences.getString("serverUrl", "").orEmpty())

    val client = HttpClient(CIO) {
        install(WebSockets)
    }

    private val dao = g.db.noteDao()

    @OptIn(ExperimentalUuidApi::class)
    suspend fun startWebsocket(host: String) {
        client.ws(method = HttpMethod.Get, host = host, port=9001, path = "/") {
            // send a message
            val sr = SyncRequests.SyncList(0u)
            send(sr.serialized())
            // receive a message
            for (frame in incoming) {
                val buffer = Buffer()
                buffer.write(frame.readBytes())
                val resp = SyncResponses.deserialize(buffer)

                when (resp) {
                    is SyncResponses.SyncList -> {
                        for (doc in resp.docs) {
                            val sr = SyncRequests.SyncDoc(0u, doc.documentId)
                            send(sr.serialized())
                        }
                    }
                    is SyncResponses.SyncDoc -> {
                        val crdt = Doc.fromInsertsAndDeletes(resp.inserts, resp.deletes)
                        println(crdt.content)
                        dao.insert(Note (
                            resp.documentId.toJavaUuid(),
                            name = resp.name,
                            crdt.display(),
                            0,
                            crdt.serialized() ))
                    }
                    else -> {}
                }
            }
        }

    }

    init {
        val editor = g.sharedPreferences.edit()
        // Observe changes to usernameState.text and debounce them
        snapshotFlow { serverUrl.text } // Convert Compose State to Kotlin Flow
            .debounce(3000L)
            .onEach { debouncedText ->
                println(debouncedText)
                editor.putString("serverUrl", debouncedText.toString())
                editor.apply()
            }
            .launchIn(viewModelScope)
    }
}