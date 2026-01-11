# Notek headless client

The idea of this client is to provide a process that handles Notek's CRDT book-keeping and server communication and let actual editors handle the text editing.

The editor connects to the client over a unix socket and all it does is send a character-level diff with this custom binary format
- ⎧ i32 1 bit for marking if the edit is a delete (1) or an insert (0) and the rest 31 bits for the index. So a -5 would signify to delete a character at 5th index and 5 would signify to insert one
  ⎩ u32 (optional, if the previous first bit is a 0) insert_char
The client then takes that information, transforms the local indeces to CRDT ones and sends them to the server. The same message applies for events received from the server and sent to the editor. The editor then takes care of inserting/deleting the characters.

Problems:
The only way that a desync could happen, would be when:
- client sends an edit with given indexes
- at the same time, before the server receives those client edits over an unix socket, there come the edits from the server
- the edits from the server get processed first
- the edits sent locally over the unix socket then get translated and applied to the document

Hence, it all depends on the latency of the unix socket communication between the editor and the headless client. In my imagination, this would be an extremely rare case, because of how the communication takes place all locally. Furthermore, this is only a concern if we use the collaborative notek, single user editing would not face that issue since there are no server edit events.
