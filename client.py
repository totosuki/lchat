import socket

# AF_INET = IPv4
# SOCK_STREAM = TCP
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

sock.connect(("127.0.0.1", 12345))

# サーバーからデータ受信
data = sock.recv(4096)

print(data)

# クライアントのソケットを削除する
sock.close()