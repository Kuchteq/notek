The sync / it's even gonna be the thing that starts the initial thing on the client:
- The client sends the last sync timestamp
- The server keeps an always ordered list in prolly some binary tree of the uids of documents to sync. When the user modifies a document, the last time of the sync the document gets removed and the current time gets put at the top.
- The server then loops over that list of document timestamps and sends the list of objects to sync. In the case where this is the first time the user is syncing with the server aka the initial sync all the elements will be sent.
- The client puts those elements in a queue, and starts to process each element:
- For each item in the queue the client sends the request to sync the document with the server. If it has the document already on the device, it looks up the last time it got modified and with that individual document sync request it sends that date, so that the server knows starting when does the client want updates.

When somebody connects for the first time they first have to decide what sort of connection this would be:
- It can either be global sync related connection and the first message you send is anything below 64
- Or it can be a start of a new editing session, then you need to first send 64 - a session greet with the document_id you want to edit. 
Once specifying the type of session, you can no longer change it.


Requests from the client:
1. synclist
- u8 header - 0
- u64 last_sync_time

2. sync_doc_pull
- u8 header - 1
- u128 document_id
- u64 last_sync_time

3. sync_doc_upsert
- u8 header - 2
- u128 document_id
- [u8] document_name - separated with a \n
- u64 last_sync_time
- u64 number_of_insert_atoms
  ⎧ u8 data_len 
  | [u8] data
  | u8 pid_depth
  | ⌈ u8  site
  ⎩ ⌊ u32 ident
- u64 number_of_delete_atoms
  ⎧ u8 pid_depth
  | ⌈ u8  site
  ⎩ ⌊ u32 ident

Responses from the server:
1. synclist_resonse
- u8 header - 32
  u64 number_of_documents
  ⎧ u64 last_sync_time
  ⎩ u128 document_id
  x number of documents

2. sync_doc_response
- u8 header - 33
- u128 document_id
- [u8] document_name - till a new line \n
- u64 number_of_insert_atoms
  ⎧ u8 data_len 
  | [u8] data
  | u8 pid_depth
  | ⌈ u8  site
  ⎩ ⌊ u32 ident
- u64 number_of_delete_atoms
  ⎧ u8 pid_depth
  | ⌈ u8  site
  ⎩ ⌊ u32 ident


Session related requests

1. session_start
- u8 header - 64
- u64 last_sync_time
- u128 document_id
Very similar to sync_doc, meant to 
a) signify that we start editing this doc and we want a session_id
b) give this final chance to sync again in case in that time between opening the app and running the sync,
something changed, tho, this might not be necessary as the background sync updates should handle that
c) Most importantly it is used to create a new document

Session related requests/responses (symmetric). Client can either send these or receive these

2. session_insert
- u8 header - 65
- u8 site
- ⎧ u8 data_len
  | [u8] data
  | u8 pid_depth
  | ⌈ u8  site
  ⎩ ⌊ u32 ident

3. session_delete
- u8 header - 66
- u8 site
- ⎧ u8 pid_depth
  | ⌈ u8  site
  ⎩ ⌊ u32 ident

4. name_change
- u8 header - 67
- [u8] document_name

- Remote has a new file:

- How does the client keep the state of affairs?
The client has a list of current documents. Each document has the id, state, name and when was the last time it was modified. It also keeps a record log of inserts and deletes, timestamped when they happened. When the user edits a document, those events get put in that log record and when they have the connection with the server, they get sent out immediately to the server one by one. Programatically, it would be done by having that log/record be a queue of events that each time a new one comes in, the websocket/event sending handler would get notified, and they would take it and dequeue if the server sucessfully received the message. This implies that the record log is at the center, not the network handler. Now if there's no connection we're offline, those logs start piling on and on, in that case, we don't want to record absolutely all the operations done to produce the final text. Just writing this sentence, I had to rephrase the sentence a couple of times and delete a lot of characters, making typos etc. We want the amount of transferred data to be exactly proportional to the new text. So in the log, if an offline user writes something and then without syncing yet, they delete it, from the perspective of the server that text has never existed if it's going to be undone immediately. So when adding a delete to the record log, we first check out if a corresponding insert is in the record log, if so, delete the insert and don't add the delete. If there's no corresponding insert, then this has to be communicated to the server. For this we will keep the record log in a hash map ig.

- How do we create a document?
There are two ways. One where the client is able to initiate a session immediately, because they're online, then it's just a session_start call. The server receives a new document_id and immediately creates a document, responds to inserts/deletes. The other one is when the client is completely offline, adds a new note, adds text etc. Then it comes online and it first does the syncdoc_upsert and then if they were in the middle of editing and they just came back online, they start a session.

Those  When there is a connection, 


The idea is we keep most of the time related things on the server?

So for each note we have three files:
- .md - text representation of the note
- .md.structure - metadata along with serialized binary representation of the document in terms of its crdt.
    > u128 document_id
    > u64 last_modified
    > binary serialized doc, i.e.
      ⎧ u8 data_len 
      | [u8] data
      | u8 pid_depth
      | ⌈ u8  site
      ⎩ ⌊ u32 ident
- .md.latest_ops - an append list of the latest x operations done on the document
