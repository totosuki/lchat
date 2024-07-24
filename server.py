import socket
import threading
from collections import defaultdict

clients = []

def launch_server() -> socket.socket:
    # AF_INET = IPv4
    # SOCK_STREAM = TCP
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.bind(("127.0.0.1", 12345))
    sock.listen()
    print("Server is listening...")
    return sock

def set_username(sock: socket.socket, client_socket: socket.socket) -> str:
    client_socket.send("None".encode("utf-8")) # 0: 名前が決まっている, 1: 名前が決まってない
    name = client_socket.recv(256).decode("utf-8")
    return name

def broadcast_message(message: str):
    for client in clients:
        try:
            client.send(message.encode("utf-8"))
        except socket.error as err:
            print(f"[Error] 送信エラー: {err}")
            client.close()
            clients.remove(client)

def listen_message(client_socket: socket.socket, name: str):
    print(f"[Log] 接続: {name}")
    while True:
        try:
            data = client_socket.recv(256).decode("utf-8")
            if not data: break
            message = f"[Chat] {name}: {data}"
            print(message)
        except socket.error as err:
            print(f"[Error] 受信エラー: {err}")
            client_socket.close()
            break
    clients.remove(client_socket)


def listen_client(sock: socket.socket):
    namedict = defaultdict(str)
    while True:
        client_socket, client_address = sock.accept()
        ip, _ = client_address
    
        if not namedict[ip]:
            namedict[ip] = set_username(sock, client_socket)
            print(f"[Server] {namedict[ip]}さんが初めて接続しました！")
            client_socket.send(f"[Server] {namedict[ip]}さんが初めて接続しました！".encode("utf-8"))
        else:
            client_socket.send(namedict[ip].encode("utf-8"))
            print(f"[Chat] {namedict[ip]}さんが接続しました！")
            client_socket.send(f"[Server] {namedict[ip]}さんが接続しました！".encode("utf-8"))
        
        clients.append(client_socket)
        client_thread = threading.Thread(target=listen_message, args=(client_socket, namedict[ip]))
        client_thread.start()

def main():
    sock = launch_server()
    listen_client(sock)

main()