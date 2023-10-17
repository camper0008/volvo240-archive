DROP TABLE IF EXISTS post;
CREATE TABLE post (
    forum_id INT NOT NULL,
    post_id INT NOT NULL,
    sub_id INT,
    title TEXT NOT NULL,
    initial_author TEXT NOT NULL,
    reply_author TEXT,
    email TEXT,
    date TEXT NOT NULL,
    initial_content TEXT NOT NULL,
    reply_content TEXT,
    corrected INT NOT NULL
);
