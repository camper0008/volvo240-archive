use rayon::prelude::*;
use regex::Regex;
use scraper::{Element, ElementRef, Html, Selector};
use serde::Serialize;
use sqlx::SqlitePool;
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicI32, Ordering},
};
use url::Url;

#[derive(Default, Serialize, Clone, Debug)]
struct Post {
    forum_id: i32,
    post_id: i32,
    title: String,
    author: String,
    email: Option<String>,
    date: String,
    content: String,
}

#[derive(Default, Serialize, Clone, Debug)]
struct Reply {
    forum_id: i32,
    post_id: i32,
    sub_id: i32,
    author: String,
    date: String,
    content: String,
}

enum Item {
    Post(Post),
    Reply(Reply),
}

static ID_COUNTER: AtomicI32 = AtomicI32::new(100000);

fn post_from_query(query: &str) -> Result<Post, String> {
    let mut post = Post::default();

    let query = &query
        .to_lowercase()
        .replace("%26amp;", "&")
        .replace("%3f", "?")
        .replace("%3d", "=")
        .replace("%26", "&")
        .replace("%3a", ":")
        .replace("%2f", "/")
        .replace("../volvo240.dk/", "");
    let query = query.parse::<Url>().unwrap();
    let query = query.query_pairs();

    for (key, value) in query {
        match key.to_lowercase().as_str() {
            "forumid" => post.forum_id = value.parse().unwrap(),
            "id" => post.post_id = value.parse().unwrap(),
            "f" if value != "4" => return Err("forum != 4".to_string()),
            "f" | "showsub" => {}
            key => return Err(format!("unrecognized key {key}")),
        }
    }

    Ok(post)
}

fn parse_link_id(child: &ElementRef) -> Option<i32> {
    let id_regex =
        Regex::new(r"[Ss][Hh][Oo][Ww][Ss][Uu][Bb]=(\d+)").expect("should be valid regex");

    let link_selector = Selector::parse("a").expect("should be valid selector");
    child
        .select(&link_selector)
        .next()
        .map(|element| {
            element
                .value()
                .attr("href")
                .map(|haystack| {
                    id_regex
                        .captures(haystack)
                        .map(|capture| {
                            capture
                                .get(1)
                                .map(|id| haystack[id.range()].parse::<i32>().ok())
                                .flatten()
                        })
                        .flatten()
                })
                .flatten()
        })
        .flatten()
}

fn reformat_date(date: String) -> Option<String> {
    let re = Regex::new(r"(?<day>\d{2})-(?<month>\d{2})-(?<year>\d{4})\s\s?(?<time>\d{2}:\d{2})").expect("should be valid regex");
    let captures = re.captures(&date)?;
    let day = &date[captures.name("day")?.range()];
    let month = &date[captures.name("month")?.range()];
    let year = &date[captures.name("year")?.range()];
    let time = &date[captures.name("time")?.range()];
    Some(format!("{year}-{month}-{day} {time}"))
}

fn parse_li_element<'a>(
    forum_id: i32,
    post_id: i32,
    child: &ElementRef,
) -> Result<(usize, Option<Reply>), String> {
    let mut elements_skipped = 0;
    let author_selector = Selector::parse("em").map_err(|_| "invalid author selector")?;

    let author_regex =
        Regex::new(r"\((.*?), (\d{2}-\d{2}-\d{4} \d{2}:\d{2})\)").expect("should be valid regex");

    let author_text = child
        .select(&author_selector)
        .next()
        .map(|v| v.text().fold(String::new(), |acc, curr| acc + curr));

    let author_match = author_text
        .as_ref()
        .map(|v| author_regex.captures(v))
        .flatten()
        .ok_or_else(|| "invalid author info".to_string())?;

    let author = author_match
        .get(1)
        .map(|v| v.range())
        .map(|range| author_text.as_ref().map(|text| (&text[range]).to_string()))
        .flatten()
        .ok_or_else(|| "reply missing author".to_string())?;

    let date = author_match
        .get(2)
        .map(|v| v.range())
        .map(|range| author_text.as_ref().map(|text| (&text[range]).to_string()))
        .flatten()
        .map(|date| reformat_date(date))
        .flatten()
        .ok_or_else(|| "reply missing date".to_string())?;

    let sub_id = parse_link_id(&child).unwrap_or_else(|| ID_COUNTER.fetch_add(1, Ordering::SeqCst));
    let Some(next) = child.next_sibling_element() else {
        return Ok((elements_skipped, None));
    };
    elements_skipped += 1;
    match next.value().name().to_lowercase().as_str() {
        "li" => {
            let (parsed_skipped_elements, parsed_post) =
                parse_li_element(forum_id, post_id, &next)?;
            return Ok((parsed_skipped_elements + elements_skipped, parsed_post));
        }
        "br" => elements_skipped += 1,
        element => return Err(format!("unhandled element: <{element}>")),
    }

    let content = next
        .next_sibling_element()
        .map(|element| {
            element.text().fold(String::new(), |acc, v| {
                acc.trim().to_string() + "\n\n" + v.trim()
            })
        })
        .map(|element| element.trim().to_string())
        .ok_or_else(|| "reply missing content")?;

    elements_skipped += 1;

    let closing_br = next
        .next_sibling_element()
        .map(|element| element.next_sibling_element())
        .flatten()
        .map(|element| element.value().name());

    match closing_br {
        Some(name) if &name.to_lowercase() == "br" => (),
        Some(name) => return Err(format!("expected <br>, got {name}")),
        None => return Err("expected <br>, got None".to_string()),
    };

    elements_skipped += 1;

    Ok((
        elements_skipped,
        Some(Reply {
            forum_id,
            post_id,
            sub_id,
            author,
            date,
            content,
        }),
    ))
}
fn parse_sub_reply_content(
    forum_id: i32,
    post_id: i32,
    reply_content: ElementRef,
) -> Result<Vec<Reply>, String> {
    /*
        message follows format:
        <td> <font> <ul>
            <li> <strong> ... <em> (author, date) </em> </strong> </li>
            <br>
            <font> content </font>
            <br>
        </ul> </font> </td>

        message is sometimes format:
        <td> <font> <ul>
            <li> <strong> <a href="/...some_url..."> </a> <em>(author, date)</em> </li>
            <br>
            <font> content </font>
            <br>
        </ul> </font> </td>
    */
    let mut posts = Vec::new();
    let mut next_child = reply_content
        .first_element_child()
        .ok_or_else(|| "no <td>".to_string())?
        .first_element_child()
        .ok_or_else(|| "no <font>".to_string())?
        .first_element_child()
        .ok_or_else(|| "no <ul>".to_string())?
        .first_element_child();
    loop {
        let Some(child) = next_child else {
            break;
        };
        match child.value().name().to_lowercase().as_str() {
            "li" => match parse_li_element(forum_id, post_id, &child)? {
                (skipped, Some(new_post)) => {
                    posts.push(new_post);
                    for _ in 0..skipped {
                        next_child = next_child
                            .map(|element| element.next_sibling_element())
                            .flatten();
                    }
                }
                (_, None) => break,
            },
            name => {
                return Err(format!("unhandled element: <{name}>"));
            }
        }
    }

    Ok(posts)
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
        .fold(String::new(), |acc, text| acc + "\n" + text)
        .trim()
        .to_string();

    if value.is_empty() {
        return Err(path.to_string() + error);
    };

    Ok(value)
}

fn process_file(path: &str) -> Result<Option<Vec<Item>>, String> {
    if path.ends_with(".jpg") || path.ends_with(".png") || path.ends_with(".gif") {
        return Ok(None);
    }

    if !path.to_lowercase().contains("f%3d4") {
        return Ok(None);
    }

    let mut post = match post_from_query(path) {
        Ok(post) => post,
        Err(_) => return Ok(None),
    };

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
        .trim()
        .to_string();
    post.author = next_row_element(&mut rows, path, ": no author")?
        .strip_prefix("Navn:")
        .ok_or_else(|| path.to_string() + ": author didn't have 'Navn:' prefix")?
        .trim()
        .to_string();
    let email_or_date = next_row_element(&mut rows, path, ": no email_or_date")?;
    if email_or_date.starts_with("E-mail:") {
        post.email = Some(email_or_date.strip_prefix("E-mail:").unwrap().to_string());
        post.date = next_row_element(&mut rows, path, ": no date")?
            .strip_prefix("Dato:")
            .and_then(|v| reformat_date(v.trim().to_string()))
            .ok_or_else(|| path.to_string() + ": date didn't have 'Dato:' prefix (a)")?;
    } else {
        post.date = email_or_date
            .strip_prefix("Dato:")
            .and_then(|v| reformat_date(v.trim().to_string()))
            .ok_or_else(|| path.to_string() + ": date didn't have 'Dato:' prefix (b)")?;
    }
    post.content = next_row_element(&mut rows, path, ": no initial_content")?;

    next_row_element(&mut rows, path, ": no reply header")?
        .strip_prefix("Svar:")
        .ok_or_else(|| path.to_string() + ": reply header didn't have 'Svar:' prefix")?;

    let  replies = parse_sub_reply_content(
        post.forum_id,
        post.post_id,
        rows.next()
            .ok_or_else(|| path.to_string() + ": no reply content")?,
    )
    .map_err(|err| path.to_string() + ": " + &err)?;

    Ok(Some(
        replies
            .into_iter()
            .map(Item::Reply)
            .chain(std::iter::once(Item::Post(post)))
            .collect(),
    ))
}

async fn scrape_content() -> io::Result<Vec<Item>> {
    let paths: Vec<_> = recurse("../volvo240.dk");
    println!("scraping...");
    println!("failed items:");
    let content = Ok(paths
        .par_iter()
        .map(|entry| process_file(entry.to_str().expect("invalid path: {entry:?}")))
        .filter_map(|entry| match entry {
            Ok(Some(post)) => Some(post),
            Ok(None) => {
                None
            },
            Err(err) => {
                eprintln!("{err}");
                None
            }
        })
        .flatten()
        .collect());
    println!("scraping completed");

    content
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let items = scrape_content().await?;
    let pool = SqlitePool::connect(env!("DATABASE_URL")).await.unwrap();
    sqlx::query!("DELETE FROM post;",)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM reply;",)
        .execute(&pool)
        .await
        .unwrap();

    println!("inserting keys...");
    println!("failed items:");
    for item in items {
        match item {
            Item::Post(post) => {
                let result = sqlx::query!(
                    "INSERT INTO post (forum_id, post_id, title, author, email, date, content) VALUES (?, ?, ?, ?, ?, ?, ?);", 
                    post.forum_id, 
                    post.post_id, 
                    post.title, 
                    post.author, 
                    post.email, 
                    post.date, 
                    post.content, 
                )
                .execute(&pool)
                .await;

                match result {
                    Ok(_) => (),
                    Err(err) if err.to_string().contains("UNIQUE constraint failed") => {},
                    Err(err) => eprintln!("forum={}, post={}: {err}", post.forum_id, post.post_id),
                }
            }
            Item::Reply(reply) => {
                let result = sqlx::query!(
                    "INSERT INTO reply (forum_id, post_id, sub_id, author, date, content) VALUES (?, ?, ?, ?, ?, ?);", 
                    reply.forum_id, 
                    reply.post_id, 
                    reply.sub_id, 
                    reply.author, 
                    reply.date, 
                    reply.content, 
                )
                .execute(&pool)
                .await;
                match result {
                    Ok(_) => (),
                    Err(err) if err.to_string().contains("UNIQUE constraint failed") => {},
                    Err(err) => eprintln!("{err}"),
                }
            },
        };
    }
    println!("insertion completed");

    Ok(())
}
