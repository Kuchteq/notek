Making a sort of efficient-ish cursor keeper.

Simple, on each operation, store the pid of where it happened, both in terms of the index and the pid, with the next operation, most likely happening next to it as in the case of regular next-to-each-other appends and deletes and go left/right from there based on the previous index and the new one. Time complexity should will be at worst O(n) 
