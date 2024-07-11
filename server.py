import socket

def launch_server() -> socket.socket:
  # AF_INET = IPv4
  # SOCK_STREAM = TCP
  sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
  sock.bind(("127.0.0.1", 12345))
  sock.listen()
  print("Server is listening...")
  return sock

def listen_client(sock: socket.socket):
  while True:
    conn, addr = sock.accept()
    print(f"Source IP Address: {addr}")
    conn.send(b"Hello World!")

def main():
  sock = launch_server()
  listen_client(sock)

main()