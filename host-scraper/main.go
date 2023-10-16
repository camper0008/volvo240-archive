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
	"strings"
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

type ForumPostList struct {
	Items        []PostItem
	Page         int
	PreviousPage int
	NextPage     int
	Limit        int
	Length       int
	ForumId      int
	Name         string
}

type PostItem struct {
	ForumId   int
	PostId    int
	Name      string
	Corrected bool
}

type Post struct {
	ForumTitle     string
	ForumId        int
	PostId         int
	SubId          int
	Title          string
	Author         string
	Email          sql.NullString
	Date           string
	InitialContent string
	ReplyContent   sql.NullString
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

func getMainPost(db *sql.DB, forum int, id int) (Post, error) {
	var mainPost Post

	err := db.QueryRow("SELECT title, author, date, forum_id, post_id, initial_content, reply_content FROM post WHERE forum_id=? AND post_id=?", forum, id).Scan(&mainPost.Title, &mainPost.Author, &mainPost.Date, &mainPost.ForumId, &mainPost.PostId, &mainPost.InitialContent, &mainPost.ReplyContent)

	mainPost.ForumTitle = forumName(mainPost.ForumId)

	return mainPost, err

}

func getSubPosts(db *sql.DB, forum int, id int) ([]Post, error) {
	query, err := db.Query("SELECT DISTINCT sub_id, reply_content FROM post WHERE forum_id=? AND post_id=? AND sub_id IS NOT NULL", forum, id)
	if err != nil {
		return nil, err
	}
	defer query.Close()
	sub_posts := make([]Post, 0)
	for query.Next() {
		var sub_post Post
		err = query.Scan(&sub_post.SubId, &sub_post.ReplyContent)
		if err != nil {
			return nil, err
		}
		sub_posts = append(sub_posts, sub_post)
	}

	return sub_posts, err
}

func queryValue(req *http.Request, query string) (int, error) {
	return strconv.Atoi(req.URL.Query().Get(query))
}

func writeTemplate[T any](w http.ResponseWriter, templatePath string, value T) {
	temp, err := template.ParseFiles(templatePath, "templates/base.tmpl")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	w.Header().Add("Content-Type", "text/html")
	err = temp.ExecuteTemplate(w, "base", value)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

}

func forumEditPost(db *sql.DB, mutex *sync.Mutex, w http.ResponseWriter, req *http.Request) {
	mutex.Lock()
	defer mutex.Unlock()

	forum, err := queryValue(req, "forum")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	post, err := queryValue(req, "post")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	if req.Method == "POST" {
		user, pass, ok := req.BasicAuth()
		if !ok || user != os.Getenv("USERNAME") || pass != os.Getenv("PASSWORD") {
			w.Header().Add("WWW-Authenticate", "Basic")
			w.WriteHeader(401)
			return
		}
		if err := req.ParseForm(); err != nil {
			w.WriteHeader(400)
			return
		}

		tx, err := db.Begin()
		defer tx.Rollback()
		if err != nil {
			w.Write([]byte("err: " + err.Error()))
			return
		}

		for key, values := range req.PostForm {
			switch key {
			case "title":
				tx.Exec("UPDATE post SET title=?, corrected=1 WHERE forum_id=? AND post_id=?", values[0], forum, post)
			case "author":
				tx.Exec("UPDATE post SET author=?, corrected=1 WHERE forum_id=? AND post_id=?", values[0], forum, post)
			case "initial-content":
				tx.Exec("UPDATE post SET initial_content=?, corrected=1 WHERE forum_id=? AND post_id=?", values[0], forum, post)
			case "reply-content":
				tx.Exec("UPDATE post SET reply_content=?, corrected=1 WHERE forum_id=? AND post_id=?", values[0], forum, post)
			default:
				if !strings.HasPrefix(key, "sub-reply-content-") {
					continue
				}
				sub_id := strings.TrimPrefix(key, "sub-reply-content-")
				sub_id_int, err := strconv.Atoi(sub_id)
				if err != nil {
					w.Write([]byte("err: " + err.Error()))
					return
				}
				tx.Exec("UPDATE post SET reply_content=?, corrected=1 WHERE forum_id=? AND post_id=? AND sub_id=?", values[0], forum, post, sub_id_int)
			}
		}
		err = tx.Commit()
		if err != nil {
			w.Write([]byte("err: " + err.Error()))
			return
		}
	}

	mainPost, err := getMainPost(db, forum, post)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	subPosts, err := getSubPosts(db, forum, post)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	writeTemplate[PostWithReplies](w, "templates/edit-post.tmpl", PostWithReplies{Post: mainPost, SubPosts: subPosts})
}

func forumPost(db *sql.DB, mutex *sync.Mutex, w http.ResponseWriter, req *http.Request) {
	mutex.Lock()
	defer mutex.Unlock()

	forum, err := queryValue(req, "forum")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	post, err := queryValue(req, "post")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	mainPost, err := getMainPost(db, forum, post)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	subPosts, err := getSubPosts(db, forum, post)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	writeTemplate[PostWithReplies](w, "templates/post.tmpl", PostWithReplies{Post: mainPost, SubPosts: subPosts})
}

func forumPostList(db *sql.DB, mutex *sync.Mutex, w http.ResponseWriter, req *http.Request) {
	mutex.Lock()
	defer mutex.Unlock()

	forum, err := queryValue(req, "forum")
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}
	page, err := queryValue(req, "page")
	if err != nil {
		page = 0
	}
	limit := 20
	rows, err := db.Query("SELECT DISTINCT forum_id, post_id, title, corrected FROM post WHERE forum_id=? LIMIT ? OFFSET ?", forum, limit, page*limit)
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	defer rows.Close()
	list := ForumPostList{
		Items:        make([]PostItem, limit),
		Page:         page,
		PreviousPage: max(0, page-1),
		NextPage:     page + 1,
		Limit:        limit,
		Length:       0,
		ForumId:      forum,
		Name:         forumName(forum),
	}

	for rows.Next() {
		var item PostItem
		err = rows.Scan(&item.ForumId, &item.PostId, &item.Name, &item.Corrected)
		if err != nil {
			w.Write([]byte("err: " + err.Error()))
			return
		}
		list.Items[list.Length] = item
		list.Length++
	}

	err = rows.Err()
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	writeTemplate[ForumPostList](w, "templates/forum-page.tmpl", list)

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
		var item ForumItem
		err = rows.Scan(&item.Id)
		if err != nil {
			w.Write([]byte("err: " + err.Error()))
			return
		}
		item.Name = forumName(item.Id)
		list = append(list, item)
	}
	err = rows.Err()
	if err != nil {
		w.Write([]byte("err: " + err.Error()))
		return
	}

	writeTemplate[[]ForumItem](w, "templates/all-forums.tmpl", list)

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
	http.HandleFunc("/edit", func(w http.ResponseWriter, r *http.Request) { forumEditPost(db, mutex, w, r) })
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
