package dev.kuchta.notek.notedetails

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CenterFocusWeak
import androidx.compose.material.icons.filled.EditCalendar
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material.icons.filled.Title
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable

@Composable
@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
fun NoteDetailsView(noteId: String, contentPadding: PaddingValues) {
    Column {
        TopAppBar(title = { Text("Note details") })
        ListItem(
            leadingContent = { Icon(imageVector = Icons.Default.Title,
                contentDescription = "Name",
                ) },
            headlineContent = { Text("A day in the life of a madman") }, supportingContent = { Text("Name")})
        ListItem(
            leadingContent = { Icon(imageVector = Icons.Default.CenterFocusWeak,
                contentDescription = "Document ID",
            ) },
            headlineContent = { Text("a1177473-74ca-4274-b2fc-40e19363c2c8") }, supportingContent = { Text("Document ID")})
        ListItem(
            leadingContent = { Icon(imageVector = Icons.Default.EditCalendar,
                contentDescription = "Last edited",
            ) },
            headlineContent = { Text("Tue Oct 21 04:46:49 PM EEST 2025") }, supportingContent = { Text("Last edited")})
        ListItem(
            leadingContent = { Icon(imageVector = Icons.Default.Sync,
                contentDescription = "Last synced",
            ) },
            headlineContent = { Text("Tue Oct 21 04:46:49 PM EEST 2025") }, supportingContent = { Text("Last synced")})
    }

}
