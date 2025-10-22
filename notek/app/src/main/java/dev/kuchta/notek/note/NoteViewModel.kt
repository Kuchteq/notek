package dev.kuchta.notek.note

import android.content.Context
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.kuchta.notek.Note
import dev.kuchta.notek.NotesDatabase
import dev.kuchta.notek.g
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class NoteViewModel() : ViewModel() {
    private val db = g.db
    private val dao = db.noteDao()

    private var id: Long = -1
    val title = TextFieldState()
    val description = TextFieldState()

    fun loadNote(noteId: Long) {
        id = noteId
        viewModelScope.launch(Dispatchers.IO) {
            val note = dao.getNoteById(noteId)
            withContext(Dispatchers.Main) {
                title.setTextAndPlaceCursorAtEnd(note?.title ?: "")
                description.setTextAndPlaceCursorAtEnd(note?.content ?: "")
            }
        }
    }

    fun startNote() {
        viewModelScope.launch(Dispatchers.IO) {
            id = dao.insert(Note(0, "", ""))
        }
    }

    init {
        snapshotFlow { title.text to description.text } // observe both fields
            .debounce(1000L) // shorter debounce
            .onEach { (debouncedTitle, debouncedDescription) ->
                viewModelScope.launch(Dispatchers.IO) {
                    dao.update(
                        Note(
                            id = id,
                            title = debouncedTitle.toString(),
                            content = debouncedDescription.toString()
                        )
                    )
                }
            }
            .launchIn(viewModelScope)
    }
}
