use rayon::prelude::*;
use scraper::{Html, Selector};
use serde::Serialize;
use sqlx::SqlitePool;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Default, Serialize)]
struct Post {
    forum_id: i32,
    post_id: i32,
    sub_id: Option<i32>,
    title: String,
    author: String,
    email: Option<String>,
    date: String,
    initial_content: String,
    reply_content: String,
    corrected: bool,
}

fn recurse(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else { return vec![] };
    entries
        .flatten()
        .flat_map(|entry| {
            let Ok(meta) = entry.metadata() else { return vec![] };
            if meta.is_dir() {
                return recurse(entry.path());
            }
            if meta.is_file() {
                return vec![entry.path()];
            }
            vec![]
        })
        .collect()
}

fn next_row_element<'a, 'b>(
    rows: &mut scraper::element_ref::Select<'a, 'b>,
    path: &'a str,
    error: &'a str,
) -> Result<String, String> {
    let value = rows
        .next()
        .ok_or_else(|| path.to_string() + error)?
        .text()
        .fold(String::new(), |acc, text| acc + text);
    let value = value.trim().to_string();

    if value.is_empty() {
        return Err(path.to_string() + error);
    };

    Ok(value)
}

fn process_file(path: &str) -> Result<Option<Post>, String> {
    if path.ends_with(".jpg") || path.ends_with(".png") || path.ends_with(".gif") {
        return Ok(None);
    }

    if !path.to_lowercase().contains("f%3d4") {
        return Ok(None);
    }

    let query = "https://volvo240.dk/".to_string() + &path.replace("%26amp;", "&");
    let query = query.parse::<Url>().unwrap();
    let query = query.query_pairs();

    let mut post = Post::default();

    for (key, value) in query {
        match key.to_lowercase().as_str() {
            "forumid" => post.forum_id = value.parse().unwrap(),
            "id" => post.post_id = value.parse().unwrap(),
            "showsub" => post.sub_id = Some(value.parse().unwrap()),
            key => return Err(format!("unrecognized key {key}")),
        }
    }

    let post_selector =
        Selector::parse("table[width='800'] > tbody > tr + tr + tr > td[width='798']")
            .expect("selector should be valid");

    let file =
        fs::read_to_string(path).map_err(|err| path.to_string() + ": " + &err.to_string())?;
    let document = Html::parse_document(&file);

    let post_element = document
        .select(&post_selector)
        .next()
        .ok_or(path.to_string() + ": no post")?;

    let row_selector = Selector::parse("tr").expect("selector should be valid");

    let mut rows = post_element.select(&row_selector);
    post.title = next_row_element(&mut rows, path, ": no title")?;
    post.author = next_row_element(&mut rows, path, ": no author")?;
    let email_or_date = next_row_element(&mut rows, path, ": no email_or_date")?;
    if email_or_date.starts_with("E-mail:") {
        post.email = Some(email_or_date);
        post.date = next_row_element(&mut rows, path, ": no date")?;
    } else {
        post.date = email_or_date;
    }
    post.initial_content = next_row_element(&mut rows, path, ": no initial_content")?;
    post.reply_content = next_row_element(&mut rows, path, ": no reply_content")?;

    Ok(Some(post))
}

fn scrape_content() -> io::Result<Vec<Post>> {
    let paths: Vec<_> = recurse("../volvo240.dk");
    println!("failed items:");
    Ok(paths
        .par_iter()
        .map(|entry| process_file(entry.to_str().expect("invalid path: {entry:?}")))
        .filter_map(|entry| match entry {
            Ok(post) => post,
            Err(err) => {
                println!("{err}");
                None
            }
        })
        .collect())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let posts = scrape_content()?;
    let pool = SqlitePool::connect(env!("DATABASE_URL")).await.unwrap();
    sqlx::query!("DELETE FROM post;",)
        .execute(&pool)
        .await
        .unwrap();

    for post in posts {
        sqlx::query!(
            "INSERT INTO post (forum_id, post_id, sub_id, title, author, email, date, initial_content, reply_content, corrected) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);", 
            post.forum_id, post.post_id, post.sub_id, post.title, post.author, post.email, post.date, post.initial_content, post.reply_content, post.corrected
        )
            .execute(&pool)
            .await
            .unwrap();
    }

    Ok(())
}
