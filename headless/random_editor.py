import struct
import random
import socket
import os

SOCKET_PATH = "/tmp/editor_socket.sock"  # adjust to your socket path

def generate_random_message(max_inserts=5, max_deletes=5, max_index=20):
    """
    Generates a random binary message in little-endian format:
    [u32 num_inserts][u32 idx + u32 char]*[u32 num_deletes][u32 idx]*
    """
    message = bytearray()

    # Inserts
    num_inserts = random.randint(0, max_inserts)
    message += struct.pack('<I', num_inserts)
    inserts = []
    for _ in range(num_inserts):
        insert_idx = random.randint(0, max_index)
        insert_char = random.randint(0x20, 0x7E)  # printable ASCII
        message += struct.pack('<II', insert_idx, insert_char)
        inserts.append((insert_idx, chr(insert_char)))

    # Deletes
    num_deletes = random.randint(0, max_deletes)
    message += struct.pack('<I', num_deletes)
    deletes = []
    for _ in range(num_deletes):
        delete_idx = random.randint(0, max_index)
        message += struct.pack('<I', delete_idx)
        deletes.append(delete_idx)

    return bytes(message), inserts, deletes

def send_message(message, socket_path=SOCKET_PATH):
    if not os.path.exists(socket_path):
        raise FileNotFoundError(f"Socket not found at {socket_path}")

    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
        client.connect(socket_path)
        client.sendall(message)
        print(f"Sent {len(message)} bytes to {socket_path}")

if __name__ == "__main__":
    msg, inserts, deletes = generate_random_message()

    print("Generated message:")
    print(f" Inserts ({len(inserts)}): {inserts}")
    print(f" Deletes ({len(deletes)}): {deletes}")
    print(f" Raw hex: {msg.hex()}")

    send_message(msg)
