const express = require("express");
const path = require("path");

const app = express();

const replaceUrlParams = (path) => {
    return path.replace("?", "%3f").replace("=", "%3d").replace("&", "%26");
};

const isHtml = (filename) => {
    return (
        filename.includes("main.asp") ||
        filename.includes("imgbs") ||
        filename.includes("img104") ||
        filename.includes(".asp") ||
        filename.includes(".html") ||
        filename.includes(".htm")
    );
};

app.get("*", (req, res) => {
    let url = replaceUrlParams(req.originalUrl);

    if (isHtml(url)) {
        res.set("Content-Type", "text/html");
    }
    res.sendFile(path.join(__dirname, "/volvo240.dk/", url));
});

app.use((req, res, next) => {
    next();
});

app.use(express.static("./volvo240.dk"));

console.log("Listening on port", 8080);
app.listen(8080);
