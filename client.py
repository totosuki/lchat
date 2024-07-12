import socket

def connect_server() -> socket.socket:
    # AF_INET = IPv4
    # SOCK_STREAM = TCP
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(("127.0.0.1", 12346))
    return sock

def main():
    sock = connect_server()
    sock.close()

main()