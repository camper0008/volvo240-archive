import os
from http.server import BaseHTTPRequestHandler, HTTPServer
import time

hostName = "localhost"
serverPort = 8080

class MyServer(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        binary = False
        html = False
        includeIndex = False
        if self.path.startswith("main.asp") or self.path.startswith("imgbs") or self.path.startswith("img104") or self.path.endswith(".asp") or self.path.endswith(".html") or self.path.endswith(".htm"):
            html = True
            self.send_header("Content-type", "text/html")
        elif self.path.endswith(".jpg"):
            binary = True
            self.send_header("Content-type", "image/jpeg")
        elif self.path.endswith(".ico"):
            binary = True
            self.send_header("Content-type", "image/x-icon")
        elif self.path.endswith(".gif"):
            binary = True
            self.send_header("Content-type", "image/gif")
        elif self.path.endswith(".js"):
            self.send_header("Content-type", "application/javascript")
        elif self.path.endswith(".css"):
            self.send_header("Content-type", "text/css")

        if self.path.endswith("/"):
            includeIndex = True

        self.end_headers()
        file_name = "." + self.path.replace("?", "%3f").replace("=","%3d").replace("&","%26")
        if includeIndex:
            file_name += "index.html"
        if not binary:
            self.wfile.write(load(file_name))
        else:
            self.wfile.write(load_binary(file_name))

def load(file):
    with open(file, 'r') as file_handle:
        return encode(str(file_handle.read()))

def encode(file):
    return bytes(file, 'UTF-8')

def load_binary(filename):
    with open(filename, 'rb') as file_handle:
        return file_handle.read()


if __name__ == "__main__":        
    webServer = HTTPServer((hostName, serverPort), MyServer)
    print("Server started http://%s:%s" % (hostName, serverPort))

    try:
        webServer.serve_forever()
    except KeyboardInterrupt:
        pass

    webServer.server_close()
    print("Server stopped.")
