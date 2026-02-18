import asyncio
import struct
import time
import uuid
import websockets

HOST = "ws://localhost:9001"

SESSION_START = 64
SESSION_INSERT = 65

SITE_ID = 1


def pack_u128_le(value: int) -> bytes:
    """
    Pack unsigned 128-bit integer (little-endian).
    Layout: low 64 bits first, then high 64 bits.
    """
    low = value & 0xFFFFFFFFFFFFFFFF
    high = (value >> 64) & 0xFFFFFFFFFFFFFFFF
    return struct.pack("<QQ", low, high)


def build_session_start(last_sync_time: int, document_id: int) -> bytes:
    """
    session_start (little-endian):

    u8   header (64)
    u64  last_sync_time
    u128 document_id
    """
    return (
        struct.pack("<B", SESSION_START) +
        struct.pack("<Q", last_sync_time) +
        pack_u128_le(document_id)
    )


def build_session_insert(char: str, ident: int) -> bytes:
    """
    session_insert (little-endian):

    u8   header (65)
    u8   site
    u8   data_len (4)
    [u8] data (4 bytes, padded)
    u8   pid_depth (1)
      u8   site
      u32  ident
    """
    encoded = char.encode("utf-8")

    if len(encoded) > 4:
        raise ValueError("Character encoding exceeds 4 bytes")

    padded = encoded.ljust(4, b"\x00")

    return (
        struct.pack("<B", SESSION_INSERT) +
        struct.pack("<B", SITE_ID) +
        struct.pack("<B", 4) +        # data_len
        padded +                      # raw bytes
        struct.pack("<B", 1) +        # pid_depth
        struct.pack("<B", SITE_ID) +
        struct.pack("<I", ident)
    )


async def main():
    async with websockets.connect(HOST) as ws:
        # ---- session_start ----
        last_sync_time = int(time.time())
        document_id = uuid.uuid4().int & ((1 << 128) - 1)

        start_packet = build_session_start(last_sync_time, document_id)
        await ws.send(start_packet)

        print("Sent session_start")

        # ---- send "hello world" ----
        text = "hello world"

        for i, ch in enumerate(text, start=1):
            packet = build_session_insert(ch, i)
            await ws.send(packet)
            print(f"Sent insert '{ch}' ident={i}")

            await asyncio.sleep(0.05)  # optional pacing


if __name__ == "__main__":
    asyncio.run(main())

