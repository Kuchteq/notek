package dev.kuchta.notek

import android.content.SharedPreferences
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Home
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.ui.NavDisplay
import dev.kuchta.notek.home.Home
import dev.kuchta.notek.note.NoteView
import dev.kuchta.notek.ui.theme.NotekTheme
import dev.kuchta.songsnatcher.TopLevelBackStack

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
    val topLevelBackStack = remember { TopLevelBackStack<Any>(NavDest.Home) }
    g.navStack = topLevelBackStack

    val context = LocalContext.current
//    g.sharedPreferences = context.getSharedPreferences("settings", Context.MODE_PRIVATE)

    NotekNavHost(topLevelBackStack)
}
sealed class NavDest {
    data object Home : NavDest()
    data class Note(val noteId: String) : NavDest()
}
@Composable
fun NotekNavHost(navStack: TopLevelBackStack<Any>) {
    Scaffold(modifier = Modifier.fillMaxSize(),
        bottomBar = {

//            NavigationBar {
//                NavigationBarItem(
//                    icon = { Icon(Icons.Filled.Home, contentDescription = "Search") },
//                    selected = navStack.topLevelKey is NavDest.Home,
//                    onClick = {navStack.addTopLevel(NavDest.Home)}
//                )
//            }
        },

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
                        NoteView(noteId = route.noteId, modifier = Modifier.padding(innerPadding))
                    }

                    else -> {
                        error("Unknown route: $route")
                    }
                }
            }
        )
    }
}
