import socket
from collections import defaultdict

def launch_server() -> socket.socket:
    # AF_INET = IPv4
    # SOCK_STREAM = TCP
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.bind(("127.0.0.1", 12346))
    sock.listen()
    print("Server is listening...")
    return sock

def set_username(sock: socket.socket, conn: socket.socket) -> str:
    conn.send("None".encode("utf-8")) # 0: 名前が決まっている, 1: 名前が決まってない
    name = conn.recv(1024).decode("utf-8")
    return name

def listen_client(sock: socket.socket):
    namedict = defaultdict(str)
    while True:
        conn, addr = sock.accept()
        ip, _ = addr
    
        if not namedict[ip]:
            namedict[ip] = set_username(sock, conn)
            print(f"[Chat] {namedict[ip]}さんが初めて接続しました！")
        else:
            conn.send(namedict[ip].encode("utf-8"))
            print(f"[Chat] {namedict[ip]}さんが接続しました！")

def main():
    sock = launch_server()
    listen_client(sock)

main()