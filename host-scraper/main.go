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
	Items        []ListItem
	Page         int
	PreviousPage int
	NextPage     int
	Limit        int
	Length       int
	ForumId      int
	Name         string
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
		return "Motor"
	case 2:
		return "Teknik"
	case 3:
		return "Karosseri"
	case 4:
		return "Volvo-folk imellem"
	case 5:
		return "volvo240.dk"
	case 6:
		return "Reservedele"
	case 7:
		return "Volvo 700 Serien"
	case 8:
		return "Volvo Motorsport"
	case 9:
		return "Volvo 900 Serien"
	case 10:
		return "Gearkasse"
	case 11:
		return "Bagtøj"
	case 12:
		return "Volvo 800 Serien"
	case 13:
		return "Forum"
	case 14:
		return "Brugerundersøgelse"
	case 15:
		return "Om volvo240.dk"
	case 16:
		return "Relateret Forum Indlæg"
	case 17:
		return "Volvo 200 træf"
	case 18:
		return "Volvo 200 Klub"
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
	page, err := strconv.Atoi(r.URL.Query().Get("page"))
	if err != nil {
		page = 0
	}
	limit := 20
	rows, err := db.Query("SELECT DISTINCT forum_id, post_id, title FROM post WHERE forum_id=? LIMIT ? OFFSET ?", forum, limit, page*limit)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	defer rows.Close()
	list := List{
		Items:        make([]ListItem, limit),
		Page:         page,
		PreviousPage: max(0, page-1),
		NextPage:     page + 1,
		Limit:        limit,
		Length:       0,
		ForumId:      forum,
		Name:         forumName(forum),
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
		list.Items[list.Length] = ListItem{ForumId: forumId, PostId: postId, Name: name}
		list.Length++
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
