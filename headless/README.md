# Notek headless client

## INITIAL PROTOCOL:
*This was the initial  protocol that I considered while under assumption that I could get a nice utf8 character based document offset in neovim/other editors in a fast and nice way. This proved to no be possible hence the new idea below*


The editor connects to the client over a unix socket and all it does is send a character-level diff with this custom binary format
- ⎧ u32 1 bit for marking if the edit is a delete (1) or an insert (0) and the rest 31 bits for the index. So a -5 would signify to delete a character at 5th index and 5 would signify to insert one (not really, its not twos complement)
  ⎩ u32 (optional, if the previous first bit is a 0) insert_char
The client then takes that information, transforms the local indeces to CRDT ones and sends them to the server. The same message applies for events received from the server and sent to the editor. The editor then takes care of inserting/deleting the characters.

## New idea

The idea of this client is to provide a process that handles the editor's buffer copy as well as Notek's CRDT book-keeping and server communication and let actual editors handle just the text editing with minimal knowledge on how notek synchronization works under the hood.
The editor connects to the client over a unix socket and it sends a **byte-level** diff with this custom binary format whose positions then get translated to atom level diffs:

1. Insert
- ⎧ u8 0 for marking it's an insert 
- | u32 32 bits for the starting index.
- | u32 length of the inserted text or in case of delete (leading bit 1) length of the delete
- ⎩ [u8] inserted text

2. Delete
- ⎧ u8 1 for marking it's a delete 
- | u32 32 bits for the starting index.
- ⎩ u32 length of the inserted text or in case of delete (leading bit 1) length of the delete

3. Chose document
- ⎧ u8 2 for marking we're choosing a given document with a given name
- | u32 document_name_len
- ⎩ [u8] document_name till the eof
Also performs a flush of the current document

4. Flush the current document
- ⎧ u8 3 for marking we're choosing a given document with a given name

Problems:
The only way that a desync could happen, would be when:
- client sends an edit with given indexes
- at the same time, before the server receives those client edits over an unix socket, there come the edits from the server
- the edits from the server get processed first
- the edits sent locally over the unix socket then get translated and applied to the document

Hence, it all depends on the latency of the unix socket communication between the editor and the headless client. In my imagination, this would be an extremely rare case, because of how the communication takes place all locally. Furthermore, this is only a concern if we use the collaborative notek, single user editing would not face that issue since there are no server edit events.

## Operation (for now, the non super seamless way)
- Starting
You start the client on the folder where you keep your notes. It makes the inital call to the server and pulls the changes.
- Editing an existing file/Creating a new file
Enter it with your editor, and the editor sends a **3. Chose document** message to the headless client. The client translates the document name to document_id by looking it up from the .md.structure file. If it's a new document, it will create one instead. Then the headless client knows that all the upcoming inserts and deletes belong to that document.
