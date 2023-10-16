DROP TABLE IF EXISTS post;
CREATE TABLE post (
    forum_id INT NOT NULL,
    post_id INT NOT NULL,
    sub_id INT,
    title BLOB NOT NULL,
    author BLOB NOT NULL,
    email BLOB,
    date BLOB NOT NULL,
    initial_content BLOB NOT NULL,
    reply_content BLOB NOT NULL,
    corrected INT NOT NULL
);
