package dev.kuchta.notek

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
}

@Composable
fun Notek() {
    val topLevelBackStack = remember { TopLevelBackStack<Any>(NavDest.Setup) }
    g.navStack = topLevelBackStack

    val context = LocalContext.current
//    g.sharedPreferences = context.getSharedPreferences("settings", Context.MODE_PRIVATE)

    NotekNavHost(topLevelBackStack)
}
sealed class NavDest {
    data object Home : NavDest()
    data class Note(val noteId: String) : NavDest()
    data class NoteDetails(val noteId: String) : NavDest()
    data object Setup : NavDest()
}
@Composable
fun NotekNavHost(navStack: TopLevelBackStack<Any>) {
    Scaffold(modifier = Modifier.fillMaxSize(),
        floatingActionButton = {
            when (navStack.backStack.last()) {
                is NavDest.Note -> NoteActionBar()
            }
            when (navStack.backStack.last()) {
                is NavDest.Setup -> ExtendedFloatingActionButton(
                    onClick = {},
                    modifier = Modifier.imePadding(),
                    icon = { Icon(Icons.Filled.Edit, "Extended floating action button.") },
                    text = { Text(text = "Save") },
                )}
//            NavigationBar {
//                NavigationBarItem(
//                    icon = { Icon(Icons.Filled.Home, contentDescription = "Search") },
//                    selected = navStack.topLevelKey is NavDest.Home,
//                    onClick = {navStack.addTopLevel(NavDest.Home)}
//                )
//            }
        },
        floatingActionButtonPosition = when (navStack.backStack.last()) {
            is NavDest.Note -> FabPosition.Center
            else -> FabPosition.End
        }

        ) { innerPadding ->

        NavDisplay(
            backStack = navStack.backStack,
            onBack = { navStack.removeLast() },
            entryProvider = { route ->
                when (route) {
                    is NavDest.Home -> NavEntry(route) {
                        Home( modifier = Modifier.padding(innerPadding) )
                    }
                    is NavDest.Note -> NavEntry(route) {
                        NoteView(noteId = route.noteId, contentPadding = innerPadding)
                    }
                    is NavDest.NoteDetails -> NavEntry(route) {
                        NoteDetailsView(noteId = route.noteId, contentPadding = innerPadding)
                    }
                    is NavDest.Setup -> NavEntry(route) {
                        SetupView(innerPadding)
                    }

                    else -> {
                        error("Unknown route: $route")
                    }
                }
            }
        )
    }
}
