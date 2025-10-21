package dev.kuchta.notek.note

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.BottomAppBar
import androidx.compose.material3.BottomAppBarDefaults.exitAlwaysScrollBehavior
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FabPosition
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.FloatingActionButtonDefaults
import androidx.compose.material3.FloatingToolbarDefaults
import androidx.compose.material3.HorizontalFloatingToolbar
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dev.kuchta.notek.g

@Composable
@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
fun NoteView(noteId: String, contentPadding: PaddingValues) {
    var title by remember { mutableStateOf("") }
    var text by remember { mutableStateOf("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Phasellus posuere feugiat odio, quis dignissim nunc dignissim nec. Etiam nec massa dolor. Duis vitae nisl posuere, lobortis enim et, dignissim massa. Duis nec erat ipsum. Vestibulum rutrum felis libero, ac tempus arcu pharetra nec. Aliquam quam ex, blandit id vestibulum et, bibendum et orci. Quisque viverra quam ante, vitae ullamcorper magna finibus et.\n" +
            "\n" +
            "Fusce ut lorem dictum, fermentum mi eu, pellentesque enim. Donec venenatis fringilla erat, ac porta justo pretium non. Aenean et metus egestas, pretium tortor ut, laoreet enim. Curabitur elit nisl, consectetur ut ante sit amet, pretium interdum arcu. Quisque in metus est. Pellentesque lobortis odio in orci aliquet, ac porttitor dui tristique. Suspendisse a ex justo. Donec erat velit, accumsan in libero nec, fermentum cursus dui." +
        "\n" +
    "In rutrum sodales purus, vel ullamcorper nisi auctor et. Curabitur hendrerit pellentesque nisi, vel rhoncus enim blandit et. Maecenas eleifend dui eu lobortis dignissim. Mauris fringilla enim consequat nulla venenatis rutrum. Proin nec odio nec mauris luctus bibendum quis a erat. Sed sed euismod felis, vel dictum lectus. In molestie velit non massa sagittis dapibus. Suspendisse pharetra mattis purus, eget scelerisque lacus mollis ut. Curabitur pretium enim nec dui eleifend, et elementum enim commodo. Aliquam ac auctor turpis. Integer in lectus dolor. Vivamus mi lorem, faucibus at egestas ac, dictum quis lorem. Integer gravida quam sed dui semper lobortis. Pellentesque aliquam erat nec nisl pulvinar, ac finibus risus euismod.") }

        Column(
            modifier = Modifier.verticalScroll(rememberScrollState())
                .padding(contentPadding)
                .consumeWindowInsets(contentPadding)
                .imePadding()
        ) {
            TextField(
                value = title,
                onValueChange = { title = it },
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
                value = text,
                onValueChange = { text = it },
                modifier = Modifier.fillMaxSize(),
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
