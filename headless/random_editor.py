import socket
import struct
import time

# Path to your Unix socket
SOCKET_PATH = "/tmp/editor_socket.sock"

def pack_edit(index: int, delete: bool, char: str = None) -> bytes:
    """
    Pack an edit command according to the protocol:
    - 1 bit op (delete=1, insert=0)
    - 31 bits index
    - optional u32 char for insert
    """
    if index < 0 or index > 0x7FFF_FFFF:
        raise ValueError("index out of range")

    op_bit = 1 if delete else 0
    value = (op_bit << 31) | index
    data = struct.pack("<I", value)

    if not delete:
        byteEncoded = char.encode("utf-8")
        data += struct.pack("<I", len(byteEncoded))
        data += byteEncoded
    return data

def main():
    # Example edits to send
    edits = [
        (0, False, 'Jeje'),  # Insert 'H' at index 0
        (1, False, 'another'),  # Insert 'i' at index 1
        # (0, True),        # Delete character at index 0
    ]

    # Connect to Unix socket
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(SOCKET_PATH)
        for edit in edits:
            index, delete = edit[0], edit[1]
            char = edit[2] if len(edit) > 2 else None
            msg = pack_edit(index, delete, char)
            s.sendall(msg)
            print(f"Sent: {edit}")
            time.sleep(2)

if __name__ == "__main__":
    main()
