package dev.kuchta.notek.setup

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.kuchta.notek.NavDest
import dev.kuchta.notek.g
import kotlinx.coroutines.launch

@Composable
@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
fun SetupView(vm: SetupViewModel = viewModel()) {
    val scope = rememberCoroutineScope() // To launch coroutines in Compose

    Scaffold(
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = { g.navStack.addTopLevel(NavDest.Home) },
                modifier = Modifier.imePadding(),
                icon = { Icon(Icons.Filled.Edit, "Extended floating action button.") },
                text = { Text(text = "Save") },
            )
        }
    ) { contentPadding ->
        Column(
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.padding(horizontal = 16.dp)
        ) {
            TopAppBar(title = { Text("Server setup") })
            TextField(vm.serverUrl, label = {Text("Adress")}, modifier = Modifier.fillMaxWidth())
            OutlinedCard(modifier = Modifier.fillMaxSize()) {
                Button(onClick = { scope.launch { vm.startWebsocket(vm.serverUrl.text.toString()) }
                }) { Text("Ping") }

                Button(onClick = { scope.launch { g.db.noteDao().wipe() }
                }) { Text("WipeDb") }
            }
//        Box(modifier = Modifier.fillMaxSize()) {
//            Card(modifier = Modifier.fillMaxWidth().align(Alignment.BottomCenter)) {
//                TextField(tfs, label = {Text("Adress")})
//            }
//        }
        }
    }
}
