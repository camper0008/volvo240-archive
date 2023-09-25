package main

import (
	"database/sql"
	"errors"
	"fmt"
	"html/template"
	"log"
	"net/http"
	"os"
	"strconv"
	"sync"

	_ "github.com/mattn/go-sqlite3"
)

func max(a int, b int) int {
	if a < b {
		return b
	} else {
		return a
	}
}

type List struct {
	Items         []ListItem
	Offset        int
	PreviousRange int
	NextRange     int
	ForumId       int
}

type ListItem struct {
	ForumId int
	PostId  int
	Name    string
}

func listFunc(db *sql.DB, mutex *sync.Mutex, w http.ResponseWriter, r *http.Request) {
	mutex.Lock()
	defer mutex.Unlock()

	forum, err := strconv.Atoi(r.URL.Query().Get("forum"))
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	offset, err := strconv.Atoi(r.URL.Query().Get("offset"))
	if err != nil {
		offset = 0
	}
	limit := 20

	rows, err := db.Query("SELECT DISTINCT forum_id, post_id, title FROM post WHERE forum_id=? LIMIT ? OFFSET ?", forum, limit, offset)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	defer rows.Close()
	list := List{
		Items:         make([]ListItem, 0),
		Offset:        offset,
		PreviousRange: max(0, offset-limit),
		NextRange:     offset + limit,
		ForumId:       forum,
	}
	for rows.Next() {
		var forumId int
		var postId int
		var name string
		err = rows.Scan(&forumId, &postId, &name)
		if err != nil {
			w.Write([]byte("err: " + err.Error()))
			return
		}
		list.Items = append(list.Items, ListItem{ForumId: forumId, PostId: postId, Name: name})
	}
	temp, err := template.ParseFiles("list.tmpl")
	w.Header().Add("Content-Type", "text/html")
	err = temp.Execute(w, list)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	err = rows.Err()
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

}

func main() {
	db, err := sql.Open("sqlite3", "./scraper.db")
	if err != nil {
		log.Fatal(err)
	}
	mutex := &sync.Mutex{}
	defer db.Close()
	http.HandleFunc("/list", func(w http.ResponseWriter, r *http.Request) { listFunc(db, mutex, w, r) })
	err = http.ListenAndServe(":3333", nil)
	if errors.Is(err, http.ErrServerClosed) {
		fmt.Printf("server closed\n")
	} else if err != nil {
		fmt.Printf("error starting server: %s\n", err)
		os.Exit(1)
	}
}
