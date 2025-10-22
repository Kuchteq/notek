package dev.kuchta.notek.note

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.kuchta.notek.Note
import dev.kuchta.notek.g
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.UUID
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@OptIn(ExperimentalUuidApi::class, FlowPreview::class)
class NoteViewModel() : ViewModel() {
    private val db = g.db
    private val dao = db.noteDao()

    private var id: UUID = UUID(0,0)
    val name = TextFieldState()
    val content= TextFieldState()

    fun loadNote(noteId: UUID) {
        id = noteId
        viewModelScope.launch(Dispatchers.IO) {
            val note = dao.getNoteById(noteId)
            withContext(Dispatchers.Main) {
                name.setTextAndPlaceCursorAtEnd(note?.name ?: "")
                content.setTextAndPlaceCursorAtEnd(note?.content ?: "")
            }
        }
    }

    fun startNote() {
        id = UUID.randomUUID()
        viewModelScope.launch(Dispatchers.IO) {
            dao.insert(Note(id, name="", content=""))
        }
    }

    init {
        snapshotFlow { name.text to content.text } // observe both fields
            .debounce(1000L) // shorter debounce
            .onEach { (debouncedName, debouncedContent) ->
                viewModelScope.launch(Dispatchers.IO) {
                    dao.update(
                        Note(
                            id = id,
                            name = debouncedName.toString(),
                            content = debouncedContent.toString()
                        )
                    )
                }
            }
            .launchIn(viewModelScope)
    }
}
