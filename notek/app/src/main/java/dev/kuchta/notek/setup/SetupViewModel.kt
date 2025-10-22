package dev.kuchta.notek.setup

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.kuchta.notek.g
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach

@OptIn(FlowPreview::class)
class SetupViewModel : ViewModel() {

    val serverUrl =
        TextFieldState(initialText = g.sharedPreferences.getString("serverUrl", "").orEmpty())

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