package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
)

func fileExists(uri string) bool {
	if _, err := os.Stat(uri); err == nil {
		return true
	}
	return false
}

func replaceUrlParams(uri string) string {
	q := strings.Replace(uri, "?", "%3f", -1)
	e := strings.Replace(q, "=", "%3d", -1)
	a := strings.Replace(e, "&", "%26", -1)
	w := strings.Replace(a, "/www/", "/", -1)
	l := strings.ToLower(w)
	return l
}

func isHtml(filename string) bool {
	return strings.Contains(filename, "imgbs") ||
		strings.Contains(filename, "img104") ||
		strings.Contains(filename, ".asp") ||
		strings.Contains(filename, ".html") ||
		strings.Contains(filename, ".htm")
}

func handle(w http.ResponseWriter, r *http.Request) {
	query := ""
	if r.URL.RawQuery != "" {
		query = "?" + r.URL.RawQuery
	}
	uri := replaceUrlParams("./volvo240.dk" + r.URL.Path + query)

	if !fileExists(uri) {
		w.Header().Set("Content-Type", "text/plain")
		fmt.Fprintf(w, "Path does not exist: %s", uri)
		return
	}

	if isHtml(uri) {
		w.Header().Set("Content-Type", "text/html")
	}
	http.ServeFile(w, r, uri)
}

func main() {
	http.HandleFunc("/", handle)
	fmt.Printf("Listening on port %d\n", 8002)
	err := http.ListenAndServe(":8002", nil)
	log.Fatalln(err)
}
