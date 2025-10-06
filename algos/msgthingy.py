import msgpack

data = {
    "type": "Greet",
}

# Write MessagePack to file
with open("greet.msgpack", "wb") as f:
    msgpack.pack(data, f)

