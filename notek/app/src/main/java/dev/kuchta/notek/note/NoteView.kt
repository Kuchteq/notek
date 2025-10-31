package dev.kuchta.notek.note

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.input.forEachChange
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.text.substring
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import java.util.UUID
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@Composable
@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class,
    ExperimentalUuidApi::class, ExperimentalFoundationApi::class
)
fun NoteView(noteId: UUID, vm : NoteViewModel = viewModel(key = noteId.toString())) {
    LaunchedEffect(noteId) {
        if (noteId == UUID(0,0)) {
            vm.startNote()
        } else {
            vm.loadNote(noteId)
        }
    }
Scaffold { contentPadding ->
    val screenHeightDp = LocalConfiguration.current.screenHeightDp.dp
    Column(
        modifier = Modifier.verticalScroll(rememberScrollState())
            .padding(contentPadding)
            .consumeWindowInsets(contentPadding)
            .imePadding()
    ) {
        TextField(
            vm.name,
            modifier = Modifier.fillMaxWidth(),
            placeholder = { Text("A note wo a name", style = MaterialTheme.typography.headlineSmall) },
            textStyle = MaterialTheme.typography.headlineSmall,
            colors = TextFieldDefaults.colors(
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent,
                disabledContainerColor = Color.Transparent,
                focusedIndicatorColor = Color.Transparent,
                unfocusedIndicatorColor = Color.Transparent,
                disabledIndicatorColor = Color.Transparent,
            )
        )
        TextField(
            vm.content,
            modifier = Modifier.fillMaxSize().heightIn(min = screenHeightDp-150.dp),
            inputTransformation = {
                val insertedChars = mutableListOf<Pair<Int, Char>>()
                val deletedSpaces = mutableListOf<Int>()
                val text = asCharSequence() // snapshot of current text once

                changes.forEachChange { sourceRange, replacedRange ->
                    val sourceStart = sourceRange.min
                    val sourceLen = sourceRange.length
                    val replacedStart = replacedRange.min
                    val replacedLen = replacedRange.length

                    val newSegment = if (sourceLen > 0) text.substring(
                        sourceStart, sourceStart + sourceLen ) else ""

                    val iterations = maxOf(replacedLen, sourceLen)
                    for (offset in 0 until iterations) {
                        if (offset < replacedLen) {
                            deletedSpaces.add(replacedStart + offset)
                        }

                        if (offset < sourceLen) {
                            newSegment.getOrNull(offset)?.let { ch ->
                                val insertPos = replacedStart + offset
                                insertedChars.add(insertPos to ch)
                            }
                        }
                    }
                }
                for (x in insertedChars) {
                    vm.localToCrdtInsert(x.first, x.second)
                }
                for (x in deletedSpaces) {
                    vm.localToCrdtDelete(x+1)
                }
                println("inserted: $insertedChars, deleted: $deletedSpaces")
            },
            colors = TextFieldDefaults.colors(
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent,
                disabledContainerColor = Color.Transparent,
                focusedIndicatorColor = Color.Transparent,
                unfocusedIndicatorColor = Color.Transparent,
                disabledIndicatorColor = Color.Transparent,
            )
        )
    }
}
}
