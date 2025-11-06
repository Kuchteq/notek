# Notek - conflict free, offline or insta-synced, collaborative notes with efficient message exchange protocol

Notek's trying to do one thing and do it well — provide access and means of editing your markdown notes across your devices in the most optimal way.

### Optimal, how?
- Notek clients communicate updates about the document with a very simple, custom, tightly packed binary format. Check out [protocol.md](/protocol.md).
- Notek's clients are native. TUI app written in Rust, a Kotlin Jetpack Compose app for Android and (soon) SwiftUI one for iOS
- The server is written in Rust.

### Base data structure

For now, the main data structure backing Notek's offline-first and collaborative capabilities is a b tree map that keeps logoot position identifiers as keys and characters as values. Logoot is not the best algorithm, but it's easy enough to hand roll in all of the languages that the clients are written in. Eventually, I'd like to use LSEQ to generate the position ids.
