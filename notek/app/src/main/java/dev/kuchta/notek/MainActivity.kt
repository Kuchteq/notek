package dev.kuchta.notek

import android.content.Context
import android.content.SharedPreferences
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Save
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.FabPosition
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.ui.NavDisplay
import dev.kuchta.notek.home.Home
import dev.kuchta.notek.note.NoteActionBar
import dev.kuchta.notek.note.NoteView
import dev.kuchta.notek.notedetails.NoteDetailsView
import dev.kuchta.notek.setup.SetupView
import dev.kuchta.notek.ui.theme.NotekTheme
import dev.kuchta.songsnatcher.TopLevelBackStack
import java.util.NavigableSet

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            NotekTheme {
                Notek()
            }
        }
    }
}
object g {
    lateinit var navStack: TopLevelBackStack<Any>
    lateinit var sharedPreferences: SharedPreferences
    lateinit var db: NotesDatabase
}

@Composable
fun Notek() {
    val context = LocalContext.current
    g.sharedPreferences = context.getSharedPreferences("settings", Context.MODE_PRIVATE)
    var startingView : NavDest = NavDest.Home

    if (g.sharedPreferences.getString("serverUrl", "").orEmpty().isEmpty()) {
        startingView = NavDest.Setup
    }

    val topLevelBackStack = remember { TopLevelBackStack<Any>(startingView) }
    g.navStack = topLevelBackStack

    g.db = NotesDatabase.getDatabase(context)
    NotekNavHost(topLevelBackStack)
}
sealed class NavDest {
    data object Home : NavDest()
    data class Note(val noteId: Long) : NavDest()
    data class NoteDetails(val noteId: String) : NavDest()
    data object Setup : NavDest()
}
@Composable
fun NotekNavHost(navStack: TopLevelBackStack<Any>) {

        NavDisplay(
            backStack = navStack.backStack,
            onBack = { navStack.removeLast() },
            entryProvider = { route ->
                when (route) {
                    is NavDest.Home -> NavEntry(route) {
                        Home()
                    }
                    is NavDest.Note -> NavEntry(route) {
                        NoteView(noteId = route.noteId)
                    }
                    is NavDest.NoteDetails -> NavEntry(route) {
                        NoteDetailsView(noteId = route.noteId)
                    }
                    is NavDest.Setup -> NavEntry(route) {
                        SetupView()
                    }

                    else -> {
                        error("Unknown route: $route")
                    }
                }
            }
        )
}
