// [archive_url, url]
// url format: http://web.archive.org/web/<timestamp>id_/<url>

use std::{fs, time::Duration};

use reqwest::blocking::Client;

fn main() {
    let sleep_seconds = 5;
    let panic_sleep_seconds = 20;
    let mut panics = 0;

    let _ = fs::create_dir("downloaded_files");
    let mut errors = Vec::new();
    let existing_files: Vec<_> = fs::read_dir("downloaded_files")
        .unwrap()
        .map(|v| v.unwrap().file_name().to_str().unwrap().to_string())
        .collect();
    let files = include_str!("transformed_file_list.json");
    let files: Vec<[String; 3]> = serde_json::from_str(files).unwrap();
    let len = files
        .iter()
        .filter(|[_, original_url, sha256]| {
            !original_url.contains("?f=5")
                && !original_url.contains("?f=3")
                && !existing_files.contains(&sha256)
        })
        .count();
    files
        .into_iter()
        .filter(|[_, original_url, sha256]| {
            !original_url.contains("?f=5")
                && !original_url.contains("?f=3")
                && !existing_files.contains(&sha256)
        })
        .enumerate()
        .map(|(i, [url, original_url, sha256])| {
            let original_url = original_url
                .strip_prefix("http://www.volvo240.dk:80")
                .or_else(|| original_url.strip_prefix("http://www.volvo240.dk"))
                .or_else(|| original_url.strip_prefix("http://volvo240.dk:80"))
                .or_else(|| original_url.strip_prefix("http://volvo240.dk"))
                .unwrap_or_else(|| &original_url);
            let client = Client::new();
            let response = match client.get(url).send() {
                Ok(v) => v,
                Err(v) => {
                    panics += 1;
                    println!(
                        "{i}/{len} -> {:.2}% ({original_url}, panic, {sha256})",
                        (((i - panics) as f32 / (len - panics) as f32) * 100.),
                        sha256 = &sha256[0..8],
                    );
                    std::thread::sleep(Duration::from_secs(panic_sleep_seconds));
                    return Err((sha256, 500, v.to_string().bytes().collect()));
                }
            };
            let status = response.status().as_u16();
            println!(
                "{i}/{len} -> {:.2}% ({original_url}, {status}, {sha256})",
                (((i - panics) as f32 / (len - panics) as f32) * 100.),
                sha256 = &sha256[0..8],
            );
            let text = response.bytes().unwrap();
            std::thread::sleep(Duration::from_secs(sleep_seconds));
            if status >= 400 {
                Err((sha256, status, text.to_vec()))
            } else {
                Ok((sha256, text))
            }
        })
        .for_each(|v| match v {
            Ok((filename, text)) => {
                fs::write(format!("downloaded_files/{filename}"), text).unwrap()
            }
            Err(err) => errors.push(err),
        });
    let files = serde_json::to_string_pretty(&errors).unwrap();
    fs::write("src/bin/errors.json", files).unwrap();
}
