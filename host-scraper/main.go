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
	Name          string
}

type ListItem struct {
	ForumId int
	PostId  int
	Name    string
}

type Post struct {
	ForumTitle     string
	ForumId        int
	PostId         int
	SubId          int
	Title          string
	Author         string
	Email          string
	Date           string
	InitialContent string
	ReplyContent   string
	Corrected      bool
}

type PostWithReplies struct {
	Post     Post
	SubPosts []Post
}

type ForumItem struct {
	Id   int
	Name string
}

func forumName(forumId int) string {
	switch forumId {
	case 1:
		return "Test"
	default:
		return "Ukendt forum"
	}
}

func forumPost(db *sql.DB, mutex *sync.Mutex, w http.ResponseWriter, r *http.Request) {
	mutex.Lock()
	defer mutex.Unlock()

	forum, err := strconv.Atoi(r.URL.Query().Get("forum_id"))
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	id, err := strconv.Atoi(r.URL.Query().Get("post_id"))
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	main_post, err := db.Query("SELECT title, author, date, forum_id, post_id, initial_content, reply_content FROM post WHERE forum_id=? AND post_id=? AND sub_id IS NULL", forum, id)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	defer main_post.Close()

	main_post_obj := Post{}
	for main_post.Next() {
		var title string
		var author string
		var date string
		var forumId int
		var postId int
		var initialContent string
		var replyContent string
		err = main_post.Scan(&title, &author, &date, &forumId, &postId, &initialContent, &replyContent)
		if err != nil {
			w.Write([]byte("err: " + err.Error()))
			return
		}
		main_post_obj.Title = title
		main_post_obj.Author = author
		main_post_obj.Date = date
		main_post_obj.InitialContent = initialContent
		main_post_obj.PostId = postId
		main_post_obj.ForumId = forumId
		main_post_obj.ForumTitle = forumName(forumId)
		main_post_obj.ReplyContent = replyContent
	}

	sub_posts, err := db.Query("SELECT DISTINCT sub_id, reply_content FROM post WHERE forum_id=? AND post_id=? AND sub_id IS NOT NULL", forum, id)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	defer sub_posts.Close()
	sub_posts_obj := make([]Post, 0)
	for sub_posts.Next() {
		sub_post := Post{}
		var subId int
		var replyContent string
		err = sub_posts.Scan(&subId, &replyContent)
		if err != nil {
			w.Write([]byte("err: " + err.Error()))
			return
		}
		sub_post.SubId = subId
		sub_post.ReplyContent = replyContent
		sub_posts_obj = append(sub_posts_obj, sub_post)
	}

	temp, err := template.ParseFiles("templates/post.tmpl", "templates/base.tmpl")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	w.Header().Add("Content-Type", "text/html")
	err = temp.ExecuteTemplate(w, "base", PostWithReplies{Post: main_post_obj, SubPosts: sub_posts_obj})
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	err = main_post.Err()
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
}

func forumPostList(db *sql.DB, mutex *sync.Mutex, w http.ResponseWriter, r *http.Request) {
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
		Name:          forumName(forum),
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
	temp, err := template.ParseFiles("templates/forum-page.tmpl", "templates/base.tmpl")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	w.Header().Add("Content-Type", "text/html")
	err = temp.ExecuteTemplate(w, "base", list)
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

func forumsList(db *sql.DB, mutex *sync.Mutex, w http.ResponseWriter, r *http.Request) {
	mutex.Lock()
	defer mutex.Unlock()

	rows, err := db.Query("SELECT DISTINCT forum_id FROM post")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	defer rows.Close()
	list := make([]ForumItem, 0)
	for rows.Next() {
		var forumId int
		err = rows.Scan(&forumId)
		if err != nil {
			w.Write([]byte("err: " + err.Error()))
			return
		}
		name := forumName(forumId)
		list = append(list, ForumItem{Name: name, Id: forumId})
	}
	temp, err := template.ParseFiles("templates/all-forums.tmpl", "templates/base.tmpl")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	w.Header().Add("Content-Type", "text/html")
	err = temp.ExecuteTemplate(w, "base", list)
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
	http.Handle("/assets/", http.StripPrefix("/assets/", http.FileServer(http.Dir("assets"))))
	http.HandleFunc("/post", func(w http.ResponseWriter, r *http.Request) { forumPost(db, mutex, w, r) })
	http.HandleFunc("/list", func(w http.ResponseWriter, r *http.Request) { forumPostList(db, mutex, w, r) })
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) { forumsList(db, mutex, w, r) })
	fmt.Println("running server on :3333")
	err = http.ListenAndServe(":3333", nil)
	if errors.Is(err, http.ErrServerClosed) {
		fmt.Printf("server closed\n")
	} else if err != nil {
		fmt.Printf("error starting server: %s\n", err)
		os.Exit(1)
	}
}
