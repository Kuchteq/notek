package dev.kuchta.notek

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.foundation.text.input.insert
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.websocket.*

@Composable
fun WebSocketDocComponent(
    client: HttpClient,
    modifier: Modifier = Modifier
) {
//    var doc by remember { mutableStateOf(Doc.fromString("local", 0u)) }
//    val trigger = remember { mutableStateOf(0) }

    val tfs = rememberTextFieldState(initialText = "")

    // Launch WebSocket client in Compose lifecycle
//    LaunchedEffect(Unit) {
//        client.ws(method = HttpMethod.Get, host = "192.168.1.222", port = 9001, path = "/") {
//            // Send initial greet
//            send(PeerMessage.Greet.serialize())
//
//            // Listen for incoming messages
//            for (frame in incoming) {
//                if (frame is Frame.Binary) {
//                    val message = PeerMessage.deserialize(frame.readBytes())
//                    println(message)
//                    when (message) {
//                        is PeerMessage.NewSession -> {
//                            doc = message.doc
//                            tfs.edit { replace(0, length, doc.display()) }
//                        }
//                        is PeerMessage.Delete -> {
//                            doc.delete(message.pid)
//                            tfs.edit { replace(0, length, doc.display())}
//                        }
//                        PeerMessage.Greet -> { /* no-op */ }
//                        is PeerMessage.Insert -> {
//                            doc.insert(message.pid, message.c)
//                            tfs.edit { replace(0, length, doc.display())}
//                        }
//                    }
//                }
//            }
//        }
//    }

    Column(modifier = modifier) {
        Text(text = "Live Doc Preview:")
            TextField(
                state = tfs,
                outputTransformation = OutputTransformation {
                    println(selection)
                }
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
