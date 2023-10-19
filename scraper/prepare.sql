DROP TABLE IF EXISTS post;
CREATE TABLE post (
    forum_id INT NOT NULL,
    post_id INT NOT NULL,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    email TEXT,
    date TEXT NOT NULL,
    content TEXT NOT NULL,
    corrected INT NOT NULL,
    UNIQUE(forum_id, post_id)
);

DROP TABLE IF EXISTS reply;
CREATE TABLE reply (
    forum_id INT NOT NULL,
    post_id INT NOT NULL,
    sub_id INT,
    author TEXT NOT NULL,
    date TEXT NOT NULL,
    content TEXT NOT NULL,
    corrected INT NOT NULL,
    UNIQUE(author, date, content, forum_id, post_id)
);
