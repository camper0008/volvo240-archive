use rayon::prelude::*;
use regex::{Captures, Regex};
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
    reply_content: Option<String>,
    corrected: bool,
}

fn parse_sub_reply_content(post: &mut Post, reply_content: &str) {
    let re = Regex::new(r"Re: .*? \((?<author>.*?), (?<date>\d{2}-\d{2}-\d{4} \d{2}:\d{2})\)")
        .expect("should compile successfully");
    let mut last_look = 0;
    let mut last_match: Option<Captures<'_>> = re.captures_at(reply_content, last_look);
    loop {
        let match_info: Option<Captures<'_>> = re.captures_at(reply_content, last_look);
        match match_info {
            Some(ref item) => {
                let full_match = item.get(0).expect("match 0 should always exist");
                if full_match.start() == last_look {
                    last_match = match_info;
                    last_look = full_match.end();
                    continue;
                }
                if let Some(info) = last_match {
                    match info.name("author") {
                        Some(author) => {
                            post.author = reply_content[author.range()].to_string();
                        }
                        None => (),
                    }
                    match info.name("date") {
                        Some(date) => post.date = reply_content[date.range()].to_string(),
                        None => (),
                    }
                    post.reply_content = Some(re.replace_all(reply_content, "").to_string());
                    break;
                }

                last_match = match_info;
                last_look = full_match.end();
                continue;
            }
            None => {
                if let Some(info) = last_match {
                    match info.name("author") {
                        Some(author) => post.author = reply_content[author.range()].to_string(),
                        None => (),
                    }
                    match info.name("date") {
                        Some(date) => post.date = reply_content[date.range()].to_string(),
                        None => (),
                    }
                    post.reply_content = Some(re.replace_all(reply_content, "").to_string());
                }
                break;
            }
        }
    }
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

    let query = &path
        .replace("%26amp;", "&")
        .replace("%3f", "?")
        .replace("%3d", "=")
        .replace("%26", "&")
        .replace("../", "https://")
        .to_lowercase();
    let query = query.parse::<Url>().unwrap();
    let query = query.query_pairs();

    let mut post = Post::default();

    for (key, value) in query {
        match key.to_lowercase().as_str() {
            "forumid" => post.forum_id = value.parse().unwrap(),
            "id" => post.post_id = value.parse().unwrap(),
            "showsub" => post.sub_id = Some(value.parse().unwrap()),
            "f" if value != "4" => return Ok(None),
            "f" => {}
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
    post.title = next_row_element(&mut rows, path, ": no title")?
        .strip_prefix("Emne:")
        .ok_or_else(|| path.to_string() + ": title didn't have 'Emne:' prefix")?
        .to_string();
    post.author = next_row_element(&mut rows, path, ": no author")?
        .strip_prefix("Navn:")
        .ok_or_else(|| path.to_string() + ": author didn't have 'Navn:' prefix")?
        .to_string();
    let email_or_date = next_row_element(&mut rows, path, ": no email_or_date")?;
    if email_or_date.starts_with("E-mail:") {
        post.email = Some(email_or_date.strip_prefix("E-mail:").unwrap().to_string());
        post.date = next_row_element(&mut rows, path, ": no date")?
            .strip_prefix("Dato:")
            .ok_or_else(|| path.to_string() + ": date didn't have 'Dato:' prefix")?
            .to_string();
    } else {
        post.date = email_or_date
            .strip_prefix("Dato:")
            .ok_or_else(|| path.to_string() + ": date didn't have 'Dato:' prefix")?
            .to_string();
    }
    post.initial_content = next_row_element(&mut rows, path, ": no initial_content")?;

    next_row_element(&mut rows, path, ": no reply header")?
        .strip_prefix("Svar:")
        .ok_or_else(|| path.to_string() + ": reply header didn't have 'Svar:' prefix")?;

    post.reply_content = Some(
        rows.next()
            .ok_or_else(|| path.to_string() + ": no reply_content")?
            .text()
            .fold(String::new(), |acc, text| acc + text)
            .trim()
            .to_string(),
    );

    if post.reply_content.as_ref().is_some_and(|c| c.is_empty()) {
        post.reply_content = None;
    };

    if let Some(reply_content) = post.reply_content.clone() {
        parse_sub_reply_content(&mut post, &reply_content);
    }

    post.corrected = !post.title.contains("#{INVALID_CHAR}#")
        && !post.author.contains("#{INVALID_CHAR}#")
        && !post.date.contains("#{INVALID_CHAR}#")
        && !post.initial_content.contains("#{INVALID_CHAR}#")
        && !post
            .reply_content
            .as_ref()
            .is_some_and(|v| v.contains("#{INVALID_CHAR}#") || v.contains("_"))
        && !post.title.contains("_")
        && !post.author.contains("_")
        && !post.date.contains("_")
        && !post.initial_content.contains("_");

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
