# Notek headless client

The idea of this client is to provide a process that handles Notek's CRDT book-keeping and server communication and let actual editors handle the text editing.

The editor connects to the client over a unix socket and all it does is send two arrays. One holding the inserted characters and their indices and one holding the deleted indices. The communication is done using a simple binary protocol with each message formatted as:
- u32 number_of_inserts
- ⎧ u32 insert_idx
  ⎩ u32 insert_char
- u32 number_of_deletes
  ( u32 insert_idx
The client then takes that information, transforms the local indeces to CRDT ones and sends them to the server. The same message applies for events received from the server and sent to the editor. The editor then takes care of inserting/deleting the characters.
