package dev.kuchta.notek

import SyncQueue
import android.content.Context
import android.content.SharedPreferences
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
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
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.viewmodel.compose.viewModel
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
import java.util.UUID

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

val LocalSyncQueue = compositionLocalOf<SyncQueue> {
    error("No SyncQueue provided")
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

    val syncQueue: SyncQueue = viewModel()
    syncQueue.startProcessing()
    CompositionLocalProvider(LocalSyncQueue provides syncQueue) {
        NotekNavHost(topLevelBackStack)
    }
}
sealed class NavDest {
    data object Home : NavDest()
    data class Note(val noteId: UUID) : NavDest()
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
                    is NavDest.Note -> NavEntry(route,
                        metadata = NavDisplay.transitionSpec {
                            // Slide new content up, keeping the old content in place underneath
                            slideInVertically(
                                initialOffsetY = { it },
                                animationSpec = tween(500)
                            ) togetherWith ExitTransition.KeepUntilTransitionsFinished
                        } + NavDisplay.popTransitionSpec {
                            // Slide old content down, revealing the new content in place underneath
                            EnterTransition.None togetherWith
                                    slideOutVertically(
                                        targetOffsetY = { it },
                                        animationSpec = tween(1000)
                                    )
                        } + NavDisplay.predictivePopTransitionSpec {
                            // Slide old content down, revealing the new content in place underneath
                            EnterTransition.None togetherWith
                                    slideOutVertically(
                                        targetOffsetY = { it },
                                        animationSpec = tween(1000)
                                    )
                        }) {
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
