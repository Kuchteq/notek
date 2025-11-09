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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.launch
import kotlinx.io.Buffer
import kotlinx.io.UnsafeIoApi
import kotlinx.io.unsafe.UnsafeBufferOperations
import java.util.TreeMap
import java.util.UUID
import kotlin.time.Clock.System.now
import kotlin.time.ExperimentalTime
import kotlin.uuid.ExperimentalUuidApi

@OptIn(UnsafeIoApi::class)
private fun ByteArray.asSource(): Source = Buffer().apply { UnsafeBufferOperations.moveToTail(this, this@asSource) }

fun TreeMap<Pid, Char>.neighborsFrom(
    fromPid: Pid,
    steps: Int,
    forward: Boolean
): Pair<Pid?, Pid?> {
    val iterator = if (forward)
        tailMap(fromPid, false).entries.iterator()
    else
        headMap(fromPid, false).descendingMap().entries.iterator()

    var target: MutableMap.MutableEntry<Pid, Char?>? = null
    repeat(steps) { if (iterator.hasNext()) target = iterator.next() }

    return if (forward) {
        val left = target?.key ?: fromPid
        val right = higherKey(left)
        left to right
    } else {
        val right = target?.key ?: fromPid
        val left = lowerKey(right)
        left to right
    }
}
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
        val pid = crdt.insertAtPhysicalOrder(p+1, ch)
        viewModelScope.launch(Dispatchers.IO) {
            pid?.let{
                sendQueue.enqueueInsert(pid, ch)
            }
        }
    }

    fun localToCrdtDelete(p: Int) {
        val pid = crdt.deleteAtPhysicalOrder(p+1)
        viewModelScope.launch(Dispatchers.IO) {
            sendQueue.enqueueDelete(pid)
        }
    }
    fun startNote(noteId: UUID) {
        id = noteId
        viewModelScope.launch(Dispatchers.IO) {
            var note = dao.getNoteById(noteId)
            if (note == null) {
                note = Note(id, name="", content="",
                    lastEdited = now().toEpochMilliseconds(), crdt.serialized())
                dao.insert(note)
            }
            val source = note.state.asSource()
            crdt = Doc.fromSource(source)
            name.setTextAndPlaceCursorAtEnd(note.name)
            content.setTextAndPlaceCursorAtEnd(crdt.display())

            val host = g.sharedPreferences.getString("serverUrl", "").orEmpty()
            sendQueue.processUpdates(client, host, id)
//            withContext(Dispatchers.Main) {
//                name.setTextAndPlaceCursorAtEnd(note?.name ?: "")
//                content.setTextAndPlaceCursorAtEnd(note?.content ?: "")
//            }

        }
    }

    init {
        viewModelScope.launch(Dispatchers.IO) {
            sendQueue.updates.debounce(1000L).collect({
                dao.insert(Note(id, name.text.toString(), content.text.toString(), now().toEpochMilliseconds(), crdt.serialized()))
            })
        }

        snapshotFlow { name.text } // Convert Compose State to Kotlin Flow
            .debounce(1000L)
            .onEach { debouncedText ->
                if (debouncedText.isNotBlank()) {
                    sendQueue.setNewTitle(debouncedText.toString())
                }
            }
            .launchIn(viewModelScope) // Launch this flow collection in the ViewModel's scope

    }

    fun startNote() {
        id = UUID.randomUUID()
        viewModelScope.launch(Dispatchers.IO) {
            dao.insert(Note(id, name="", content="",
                lastEdited = now().toEpochMilliseconds(), crdt.serialized()))
            startNote(id)
        }
    }
    fun onNoteExit() {
        viewModelScope.launch(Dispatchers.IO) {
            sendQueue.finish()
        }
    }

}
