import socket
import threading

def connect_server() -> socket.socket:
    # AF_INET = IPv4
    # SOCK_STREAM = TCP
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect(("127.0.0.1", 12345))
    except socket.error as err:
        print(f"[Error] サーバーに接続できませんでした: {err}")
        exit()
    return sock

def check_username(sock: socket.socket):
    # username状況を確認
    data = sock.recv(256).decode("utf-8")
    
    if data == "None":
        name = input("名前を入力してください: ")
        sock.send(name.encode("utf-8"))
    else:
        print(f"{data}さん、こんにちは")

def receive_messages(sock: socket.socket):
    while True:
        data = sock.recv(256).decode("utf-8")
        print(data)

def chat(client_socket: socket.socket):
    while True:
        try:
            message = input()
            client_socket.send(message.encode("utf-8"))
        except socket.error as err:
            print(f"[Error] 送信エラー: {err}")
            client_socket.close()
            break

def main():
    client_socket = connect_server()
    check_username(client_socket)
    
    receive_thread = threading.Thread(target=receive_messages, args=(client_socket,))
    receive_thread.start()
    
    chat(client_socket)

main()