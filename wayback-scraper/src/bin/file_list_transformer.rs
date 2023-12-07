// [url, mimetype, timestamp, endtimestamp, groupcount, uniqcount]
// url format: http://web.archive.org/web/<timestamp>id_/<url>

use std::fs;

fn main() {
    let files = include_str!("archive_file_list.json");
    let files: Vec<[String; 6]> = serde_json::from_str(files).unwrap();
    let files: Vec<_> = files
        .into_iter()
        .map(
            |[url, _mimetype, _timestamp, endtimestamp, _groupcount, _uniqcount]| {
                [
                    format!("http://web.archive.org/web/{endtimestamp}id_/{url}"),
                    url.clone(),
                    sha256::digest(&url),
                ]
            },
        )
        .collect();
    let files = serde_json::to_string(&files).unwrap();
    fs::write("src/bin/transformed_file_list.json", files).unwrap();
}
