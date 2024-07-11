import socket

# AF_INET = IPv4
# SOCK_STREAM = TCP
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

sock.bind(("127.0.0.1", 12345))

sock.listen()

print("Server is listening...")

while True:
  conn, addr = sock.accept()
  
  print(f"Source IP Address: {addr}")
  
  # クライアントのソケットにデータを送信
  conn.send(b"Hello World!")