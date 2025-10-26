package dev.kuchta.notek.sync

import Pid
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.Mutex.*

class SendQueue {
    // Char? if it's null it means that it's a Delete even we need to send
    private val map = LinkedHashMap<Pid, Char?>()
    private val _updates = MutableSharedFlow<Unit>(extraBufferCapacity = 64) // signals "something changed"
    val updates: SharedFlow<Unit> = _updates

    private val mutex = Mutex()

    suspend fun enqueue(pid: Pid, ch: Char?) {
        mutex.withLock {
            map[pid] = ch
        }
        _updates.emit(Unit) // emit only a trigger
    }

    suspend fun remove(pid: Pid) {
        mutex.withLock {
            map.remove(pid)
        }
    }
    suspend fun peekFirst(): Pair<Pid, Char?>? {
        return mutex.withLock {
            map.entries.firstOrNull()?.let { it.key to it.value }
        }
    }

    suspend fun getSnapshot(): List<Pair<Pid, Char?>> {
        return mutex.withLock { map.entries.map { it.key to it.value } }
    }

    suspend fun dequeue(pid: Pid) {
        mutex.withLock { map.remove(pid) }
    }
}
