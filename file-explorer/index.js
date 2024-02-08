const { readdir } = require("fs/promises");
const { join } = require("path");
const app = require("express")();

const port = 8080;

function isHtml (filename) {
    return (
        filename.includes("main.asp") ||
        filename.includes("imgbs") ||
        filename.includes("img104") ||
        filename.includes(".asp") ||
        filename.includes(".html") ||
        filename.includes(".htm")
    );
};

async function main() {
    console.log("listening on", port);
    const app_list = await readdir("../volvo240.dk")
        .then(entry => entry.filter(v => !v.includes("f%3D4")))
        .then(entry => entry.map(v => [v, decodeURIComponent(v)]))
        .then(entry => entry.map(([filename, url]) => [filename, new URL(url)]))
    app.get("/", (req, res) => {
        const list = app_list.map(([filename, url]) => `<a href="${url.pathname}${url.search}">${url.pathname}${url.search}</a>`)
        res.send(list.join("<br>"));
    })
    app.get("/*", (req, res) => {
        const entry = app_list.find(([_, url]) => req.originalUrl === `${url.pathname}${url.search}`)
        if (!entry) {
            res.set("Content-Type", "text/plain");
            return res.status(404).send(`ukendt side ${req.originalUrl}`)
        }
        const [filename] = entry;
        const path = join(__dirname, "../volvo240.dk", filename);

        if (isHtml(req.originalUrl)) {
            res.set("Content-Type", "text/html");
        }

        res.sendFile(path);
    })
    app.listen(port);
}

main();
