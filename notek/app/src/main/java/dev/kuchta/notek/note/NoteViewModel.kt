package dev.kuchta.notek.note

import Doc
import Pid
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.kuchta.notek.Note
import dev.kuchta.notek.g
import kotlinx.io.Source
import dev.kuchta.notek.sync.SendQueue
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.ws
import io.ktor.http.HttpMethod
import io.ktor.websocket.send
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.launch
import kotlinx.io.Buffer
import kotlinx.io.UnsafeIoApi
import kotlinx.io.unsafe.UnsafeBufferOperations
import org.example.Session
import java.util.UUID
import kotlin.time.Clock.System.now
import kotlin.time.ExperimentalTime
import kotlin.uuid.ExperimentalUuidApi

@OptIn(UnsafeIoApi::class)
private fun ByteArray.asSource(): Source = Buffer().apply { UnsafeBufferOperations.moveToTail(this, this@asSource) }
@OptIn(ExperimentalUuidApi::class, FlowPreview::class, ExperimentalTime::class)
class NoteViewModel() : ViewModel() {
    private val db = g.db
    private val dao = db.noteDao()

    private var id: UUID = UUID(0,0)
    private var crdt: Doc = Doc.empty();
    val name = TextFieldState("")
    val content = TextFieldState("")
    val sendQueue = SendQueue()

    val client = HttpClient(CIO) {
        install(WebSockets)
    }
    fun localToCrdtInsert(p: Int, ch: Char) {
        val pid = Pid.new1d(p.toUInt()+1u, 1u)
        crdt.insert(pid, ch)
        viewModelScope.launch(Dispatchers.IO) {
            sendQueue.enqueue(pid, ch)
        }
    }

    fun localToCrdtDelete(p: Int) {
        val pid = Pid.new1d(p.toUInt()+1u, 1u)
        crdt.delete(pid)
        viewModelScope.launch(Dispatchers.IO) {
            sendQueue.enqueue(pid, null)
        }
    }
    fun loadNote(noteId: UUID) {
        id = noteId
        viewModelScope.launch(Dispatchers.IO) {
            val note = dao.getNoteById(noteId)
            if (note == null) {
                return@launch
            }
            val source = note.state.asSource()
            crdt = Doc.fromSource(source)
            name.setTextAndPlaceCursorAtEnd(note.name)
            content.setTextAndPlaceCursorAtEnd(crdt.display())

            val host = g.sharedPreferences.getString("serverUrl", "").orEmpty()
            client.ws(method = HttpMethod.Get, host = host, port=9001, path = "/") {
                val sr = Session.Start(0u,noteId)
                send(sr.serialized())
                sendQueue.updates.collect({
                    // Peek at the first queued update (may be null)
                    sendQueue.peekFirst()?.let { (pid, ch) ->
                        if (ch != null) {
                            // Insert event
                            val sr = Session.Insert(0u, pid, ch)
                            send(sr.serialized())
                        } else {
                            // Delete event
                            val sr = Session.Delete(0u, pid)
                            send(sr.serialized())
                        }
                        sendQueue.dequeue(pid)
                    }
                })
            }
//            withContext(Dispatchers.Main) {
//                name.setTextAndPlaceCursorAtEnd(note?.name ?: "")
//                content.setTextAndPlaceCursorAtEnd(note?.content ?: "")
//            }

        }
    }

    fun startNote() {
        id = UUID.randomUUID()
        viewModelScope.launch(Dispatchers.IO) {
            dao.insert(Note(id, name="", content="", lastEdited = now().toEpochMilliseconds()))
        }
    }

}
