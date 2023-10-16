DROP TABLE IF EXISTS post;
CREATE TABLE post (
    forum_id INT NOT NULL,
    post_id INT NOT NULL,
    sub_id INT,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    email TEXT,
    date TEXT NOT NULL,
    initial_content TEXT NOT NULL,
    reply_content TEXT,
    corrected INT NOT NULL
);
