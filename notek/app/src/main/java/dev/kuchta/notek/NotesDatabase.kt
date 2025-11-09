package dev.kuchta.notek

import android.content.ContentValues
import android.content.Context
import androidx.room.*
import kotlinx.coroutines.flow.Flow
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid
import java.util.UUID



@Entity(tableName = "notes")
data class Note (
    @PrimaryKey()
    val id: UUID = UUID.randomUUID(),
    val name: String,
    val content: String,
    val lastEdited: Long = 0,
    val state: ByteArray = ByteArray(0)

)
@Dao
interface NoteDao {

    @Query("SELECT * FROM notes ORDER BY lastEdited DESC")
    fun getAllNotes(): Flow<List<Note>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(note: Note)

    @Update
    suspend fun update(note: Note)

    @Delete
    suspend fun delete(note: Note)

    @Query("SELECT * FROM notes WHERE id = :id")
    suspend fun getNoteById(id: UUID): Note?

    @Query("DELETE FROM notes")
    suspend fun wipe()
}

@Database(entities = [Note::class], version = 1)
abstract class NotesDatabase : RoomDatabase() {

    abstract fun noteDao(): NoteDao

    companion object {
        @Volatile
        private var INSTANCE: NotesDatabase? = null

        fun getDatabase(context: Context): NotesDatabase {
            return INSTANCE ?: synchronized(this) {
                val instance = Room.databaseBuilder(
                    context.applicationContext,
                    NotesDatabase::class.java,
                    "notes.db"
                ).build()
                INSTANCE = instance
                instance
            }
        }
    }
}