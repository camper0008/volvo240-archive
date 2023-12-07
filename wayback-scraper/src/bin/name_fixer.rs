// [archive_url, url]
// url format: http://web.archive.org/web/<timestamp>id_/<url>

use std::fs;

fn main() {
    let _ = fs::create_dir("renamed_files");
    let files = include_str!("transformed_file_list.json");
    let files: Vec<[String; 3]> = serde_json::from_str(files).unwrap();
    files
        .into_iter()
        .filter(|[_, original_url, _]| {
            !original_url.contains("?f=5") && !original_url.contains("?f=3")
        })
        .for_each(|[_, original_url, sha256]| {
            match fs::copy(
                format!("downloaded_files/{sha256}"),
                format!("renamed_files/{}", urlencoding::encode(&original_url)),
            ) {
                Ok(_) => {}
                Err(err) => println!("downloaded_files/{original_url}: {err}"),
            }
        });
}
