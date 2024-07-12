import socket

def connect_server() -> socket.socket:
    # AF_INET = IPv4
    # SOCK_STREAM = TCP
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(("127.0.0.1", 12346))
    return sock

def check_username(sock: socket.socket):
    # username状況を確認
    data = sock.recv(256).decode("utf-8")
    
    if data == "None":
        name = input("名前を入力してください: ")
        sock.send(name.encode("utf-8"))
    else:
        print(f"{data}さん、こんにちは")

def main():
    sock = connect_server()
    check_username(sock)
    sock.close()

main()