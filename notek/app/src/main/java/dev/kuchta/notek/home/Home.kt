package dev.kuchta.notek.home

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CloudDone
import androidx.compose.material.icons.filled.CloudOff
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxDefaults
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.toString
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.kuchta.notek.NavDest
import dev.kuchta.notek.g
import dev.kuchta.notek.note.HomeViewModel
import dev.kuchta.notek.setup.SetupViewModel
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import kotlin.time.ExperimentalTime
import kotlin.time.Instant
import kotlin.uuid.ExperimentalUuidApi

data class NoteOverview(
    val id: String,
    val title: String,
    val content: String,
    val lastEdited: String
)

@OptIn(ExperimentalMaterial3Api::class, ExperimentalUuidApi::class, ExperimentalTime::class)
@Composable
fun Home(vm: HomeViewModel = viewModel()) {
    // Simulated connection status (in real app, this would be from a ViewModel or state)
    val isConnected by remember { mutableStateOf(true) }
    val notes by vm.notes.collectAsState()


    Scaffold(
        floatingActionButton = {
        HomeActionBar()
        },
        topBar = { TopAppBar(title={ConnectionStatus(isConnected = isConnected) }, actions = {
            IconButton(onClick = {g.navStack.addTopLevel(NavDest.Setup)}) { Icon(Icons.Default.Settings, "")}
        })}
    ) { contentPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(contentPadding).padding(horizontal = 16.dp)
        ) {
            // 🔌 Status indicator
            Spacer(modifier = Modifier.height(16.dp))

            // 📜 Notes list
            LazyColumn(
                verticalArrangement = Arrangement.spacedBy(12.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.fillMaxSize()
            ) {
                items(notes) { note ->
                    NoteCard(NoteOverview(note.id.toString(), note.name, note.content,
                        formatEpochMillis(note.lastEdited)) ) {
                        g.navStack.add(NavDest.Note(note.id))
                    }
                }
            }
        }
    }
}

@Composable
fun ConnectionStatus(isConnected: Boolean) {
    val (icon, color, text) = if (isConnected) {
        Triple(Icons.Default.CloudDone, MaterialTheme.colorScheme.primary, "Connected")
    } else {
        Triple(Icons.Default.CloudOff, MaterialTheme.colorScheme.error, "Offline")
    }
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ){

        Icon(
            imageVector = icon,
            contentDescription = text,
            tint = color,
//            modifier = Modifier.size(20.dp)
        )
        Spacer(modifier = Modifier.width(16.dp))
        Column {
                Text(
                    text = text,
                    color = color,
                )
                Text("notek.kuchta.dev",
                    style = MaterialTheme.typography.bodyMedium )
            }
        }
}

fun formatEpochMillis(epochMillis: Long, timeZone: String = ZoneId.systemDefault().id): String {
    val instant = java.time.Instant.ofEpochMilli(epochMillis)
    val formatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss")
        .withZone(ZoneId.of(timeZone))
    return formatter.format(instant)
}
@Composable
fun NoteCard(note: NoteOverview, onClick: () -> Unit) {
    {
//            if (it == StartToEnd) onToggleDone(todoItem)
//            else if (it == EndToStart) onRemove(todoItem)
//            // Reset item when toggling done status
//            it != StartToEnd
    }
    val swipeToDismissBoxState =
        rememberSwipeToDismissBoxState(
            SwipeToDismissBoxValue.Settled,
            SwipeToDismissBoxDefaults.positionalThreshold
        )

    SwipeToDismissBox(
        state = swipeToDismissBoxState,
        backgroundContent = {
            when (swipeToDismissBoxState.dismissDirection) {
                SwipeToDismissBoxValue.EndToStart -> {
                    Icon(
                        imageVector = Icons.Default.Delete,
                        contentDescription = "Remove item",
                        modifier = Modifier
                            .fillMaxSize()
                            .background(Color.Red)
                            .wrapContentSize(Alignment.CenterEnd)
                            .padding(12.dp),
                        tint = Color.White
                    )
                }
                SwipeToDismissBoxValue.Settled -> {}
                SwipeToDismissBoxValue.StartToEnd -> {}
            }
        }
    ) {
        Card(
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surfaceVariant,
            ),
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 100.dp)
                .clickable(onClick = onClick)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(16.dp)
            ) {
                // Title + last edited row
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = note.title,
                        style = MaterialTheme.typography.titleMedium,
                        textAlign = TextAlign.Start
                    )
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            imageVector = Icons.Default.Edit,
                            contentDescription = "Last edited",
                            modifier = Modifier
                                .size(16.dp)
                                .padding(end = 4.dp)
                        )
                        Text(
                            text = note.lastEdited,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                Spacer(modifier = Modifier.height(8.dp))

                Text(
                    text = note.content,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}
