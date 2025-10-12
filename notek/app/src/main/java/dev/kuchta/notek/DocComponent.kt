package dev.kuchta.notek

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.websocket.*
import io.ktor.http.*
import io.ktor.websocket.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

@Composable
fun WebSocketDocComponent(
    client: HttpClient,
    modifier: Modifier = Modifier
) {
    var doc by remember { mutableStateOf(Doc.fromString("local", 0u)) }
    val trigger = remember { mutableStateOf(0) }

    // Launch WebSocket client in Compose lifecycle
    LaunchedEffect(Unit) {
        client.ws(method = HttpMethod.Get, host = "192.168.191.179", port = 9001, path = "/") {
            // Send initial greet
            send(PeerMessage.Greet.serialize())

            // Listen for incoming messages
            for (frame in incoming) {
                if (frame is Frame.Binary) {
                    val message = PeerMessage.deserialize(frame.readBytes())
                    println(message)
                    when (message) {
                        is PeerMessage.NewSession -> {
                            doc = message.doc
                        }
                        is PeerMessage.Delete -> {
                            doc.delete(message.pid)
                        }
                        PeerMessage.Greet -> { /* no-op */ }
                        is PeerMessage.Insert -> {
                            doc.insert(message.pid, message.c)
                        }
                    }
                }
            }
        }
    }

    Column(modifier = modifier) {
        Text(text = "Live Doc Preview:")
        TextField(
            value = doc.display(), // show the document content
            onValueChange = {},    // read-only for now
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Preview(showBackground = true)
@Composable
fun PreviewWebSocketDocComponent() {
    val client = HttpClient(CIO) {
        install(WebSockets)
    }

    WebSocketDocComponent(client)
}
