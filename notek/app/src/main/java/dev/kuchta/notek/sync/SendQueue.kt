package dev.kuchta.notek.sync

import Pid
import io.ktor.client.HttpClient
import io.ktor.client.plugins.websocket.ws
import io.ktor.http.HttpMethod
import io.ktor.websocket.CloseReason
import io.ktor.websocket.close
import io.ktor.websocket.send
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.example.Session
import java.util.UUID
sealed class SendQueueItem {
    data class Insert(val pid: Pid, val ch: Char) : SendQueueItem()
    data class Delete(val pid: Pid): SendQueueItem()
    data class ChangeName(val name: String): SendQueueItem()
    object Finish: SendQueueItem()
}

class SendQueue {
    // Char? if it's null it means that it's a Delete even we need to send
    private val charUpdateQueue = LinkedHashMap<Pid, Char?>()
    private var upcomingTitleUpdate : String? = null;
    private var shouldFinish : Boolean = false;
    private val _updates = MutableSharedFlow<Unit>(extraBufferCapacity = 64) // signals "something changed"
    val updates: SharedFlow<Unit> = _updates

    private val mutex = Mutex()

    suspend fun enqueueInsert(pid: Pid, ch: Char) {
        mutex.withLock {
            charUpdateQueue[pid] = ch
        }
        _updates.emit(Unit) // emit only a trigger
    }

    suspend fun enqueueDelete(pid: Pid) {
        mutex.withLock {
            if (charUpdateQueue.contains(pid)) {
                charUpdateQueue.remove(pid)
            }
            else {
                charUpdateQueue[pid] = null
            }
        }
        _updates.emit(Unit)
    }
    suspend fun peekFirst(): Pair<Pid, Char?>? {
        return mutex.withLock {
            charUpdateQueue.entries.firstOrNull()?.let { it.key to it.value }
        }
    }

    suspend fun getSnapshot(): List<Pair<Pid, Char?>> {
        return mutex.withLock { charUpdateQueue.entries.map { it.key to it.value } }
    }
    suspend fun setNewTitle(name: String) {
        upcomingTitleUpdate = name
        _updates.emit(Unit)
    }

    suspend fun dequeue(pid: Pid) {
        mutex.withLock { charUpdateQueue.remove(pid) }
    }
    suspend fun finish() {
        shouldFinish = true
        _updates.emit(Unit)
    }
    suspend fun processUpdates(client: HttpClient, host: String, noteId: UUID) {
        client.ws(method = HttpMethod.Get, host = host, port=9001, path = "/") {
            val sr = Session.Start(0u,noteId)
            send(sr.serialized())
            updates.collect({
                println("Update!")
                // Peek at the first queued update (may be null)
                peekFirst()?.let { (pid, ch) ->
                    if (ch != null) {
                        // Insert event
                        val sr = Session.Insert(0u, pid, ch)
                        send(sr.serialized())
                    } else {
                        // Delete event
                        val sr = Session.Delete(0u, pid)
                        send(sr.serialized())
                    }
                    dequeue(pid)
                }
                upcomingTitleUpdate?.let {
                    val sr = Session.ChangeName(upcomingTitleUpdate!!);
                    send(sr.serialized())

                    upcomingTitleUpdate = null;
                }
                if (shouldFinish) {
                    close(CloseReason(CloseReason.Codes.NORMAL, "All updates processed"))
                }
            })
        }
    }
}
