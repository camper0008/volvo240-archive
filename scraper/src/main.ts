import { readdir, readFile } from "fs/promises";
import HTMLParser from 'node-html-parser';
import { JSDOM } from "jsdom";
const sqlite3 = require("sqlite3").verbose();

// URL NOTE:
// f = template
// id = id
// forumid = forumid
// showsub = show reply

function unescape(string: string): string {
    const DOMParser = new JSDOM().window.DOMParser;
    return new DOMParser().parseFromString(string, 'text/html').documentElement.textContent ?? "";
}

function fix(value: string | null | undefined, bad: string, replacement?: string): string | null {
    if (value === null || value === undefined) {
        return null
    }
    let str = value;
    while (str.search(bad) !== -1) {
        str = str.replace(bad, replacement ?? "").trim();
    }
    return str;
}

function fix_iso_8859_encoding(value: string | null): string | null {
    if (value === null) {
        return null;
    }
    console.log(value);
    const fixes = [
        ["&nbsp;", "",],
        ["&quot;", "\"",],
        ["&amp;", "&",],
        ["\xF8", "ø",],
        ["\xE6", "æ",],
        ["\xE5", "å",],
        ["\xD8", "Ø",],
        ["\xC6", "Æ",],
        ["\xC5", "Å",],
    ]
    const is_fixed = (str: string, bad: string[]) => {
        return !bad.map(bad => str.search(bad)).some(v => v !== -1);
    }

    const bad_items = value.matchAll(/&#\d{0,3};/g);
    for (const bad_item of bad_items) {
        if (fixes.findIndex(([bad]) => bad === bad_item[0]) === -1)
            fixes.push([bad_item[0], unescape(bad_item[0])])
    }

    while (!is_fixed(value, fixes.map(([bad]) => bad))) {
        for (const [bad, good] of fixes) {
            value = value.replace(bad, good);
        }
    }
    return value;
}

async function main() {
    console.log(process.cwd() + "/scraper.db")
    const db = new sqlite3.Database(`sqlite://../scraper.db`);
    const dirs = await readdir("../volvo240.dk");
    const base_url = "https://volvo240.dk"
    const dirs_map = dirs
        .map((filename) => ({ filename, url: decodeURIComponent(filename.toLowerCase()) }))
        .map(({ filename, url }) => ({ filename, url: new URL(url, base_url) }))
        .filter(({ url }) => url.searchParams.get("f") === "4")
    let files_read = 0;
    let sub_files = 0;

    db.run("DELETE FROM post");
    const stmnt = db.prepare("INSERT INTO post(forum_id, post_id, sub_id, title, author, email, date, initial_content, reply_content, corrected) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)");

    const max_files = dirs_map.length;
    const effects = dirs_map.map(async ({ filename, url }) => {
        const content = await readFile(`../volvo240.dk/${filename.toLowerCase()}`, "utf-8");
        if (!content) {
            return filename
        }
        const root = HTMLParser.parse(content);
        const post = root.querySelector("table[width='800'] > tbody > tr + tr + tr > td[width='798']");
        if (post === null) {
            return filename
        }
        files_read += 1;
        if (url.searchParams.get("showsub")) {
            sub_files += 1;
        };
        const forum_id = url.searchParams.get("forumid");
        const post_id = url.searchParams.get("id");
        const sub_id = url.searchParams.get("showsub");

        // CONTENT NOTE:
        // order = emne -> navn -> email? -> dato -> initial_content -> svar_title -> reply_content

        const rows = post.querySelectorAll("tr").map((el) => el.innerText);
        const title = rows.shift()?.trim() ?? "_";
        console.assert(title.startsWith("Emne:"), "emne");
        const author = rows.shift()?.trim() ?? "_";
        console.assert(author.startsWith("Navn:"), "navn");
        const emailOrDate = rows.shift()?.trim() ?? "_";
        let email, date;
        if (emailOrDate.startsWith("E-mail:") || emailOrDate.startsWith("E-mail:")) {
            email = emailOrDate;
        } else {
            date = emailOrDate;
            console.assert(date.startsWith("Dato:"), "dato", date);
        }
        if (!date) {
            date = rows.shift()?.trim() ?? "_";
            console.assert(date.startsWith("Dato:"), "dato", date);
        }
        const initial_content = rows.shift()?.trim();
        rows.shift()?.trim();
        if (!initial_content) {
            return filename
        }
        const reply_content = rows.shift()?.trim();
        if (!reply_content) {
            return filename
        }

        stmnt.run(
            forum_id,
            post_id,
            sub_id,
            fix(title, "Emne:"),
            fix(author, "Navn:"),
            fix(email, "Email:"),
            fix(date, "Dato:"),
            initial_content,
            reply_content,
            0
        );

        return null;
    })
    const errors = await Promise.all(effects);
    console.log("files with errors:");
    errors.filter(errors => errors !== null).forEach(error => console.log(error));
    console.log("sub files: ", sub_files, "/", max_files);
}

main().catch(console.error);
