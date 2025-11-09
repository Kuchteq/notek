package dev.kuchta.notek.home

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FloatingToolbarDefaults
import androidx.compose.material3.HorizontalFloatingToolbar
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dev.kuchta.notek.NavDest
import dev.kuchta.notek.g
import java.util.UUID

@Composable
@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
fun HomeActionBar() {
    val vibrantColors = FloatingToolbarDefaults.vibrantFloatingToolbarColors()

    HorizontalFloatingToolbar(
        expanded = true,
        floatingActionButton = {
            // Match the FAB to the vibrantColors. See also StandardFloatingActionButton.
            FloatingToolbarDefaults.VibrantFloatingActionButton(
                onClick = {g.navStack.add(NavDest.Note(UUID.randomUUID()))}
            ) {
                Icon(Icons.Filled.Add, "Add Note")
            }
        },
        colors = vibrantColors,
        content = {
            IconButton(onClick = { /* doSomething() */ }) {
                Icon(Icons.Filled.Search, contentDescription = "Localized description")
            }
            IconButton(onClick = {
                g.navStack.add(NavDest.NoteDetails("kurwa"))
            }) {
                Icon(Icons.Filled.MoreVert, contentDescription = "Localized description")
            }
        },
    )
}
