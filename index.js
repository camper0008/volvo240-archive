const express = require("express");
const path = require("path");
const fs = require("fs");

const fileExists = (path) => {
    try {
        fs.statSync(path);
        return true;
    } catch (err) {
        return false;
    }
};

const app = express();

const replaceAllUrlParams = (path) => {
    return path
        .replaceAll("?", "%3f")
        .replaceAll("=", "%3d")
        .replaceAll("&", "%26")
        .replaceAll("/www/", "/")
        .toLowerCase();
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
    const url = replaceAllUrlParams(req.originalUrl);

    const filePath = path.join(__dirname, "/volvo240.dk/", url);

    if (!fileExists(filePath)) {
        res.set("Content-Type", "text/plain");
        res.send("Path does not exist: " + filePath);
        return;
    }

    if (isHtml(url)) {
        res.set("Content-Type", "text/html");
    }
    res.sendFile(filePath);
});

app.use((req, res, next) => {
    next();
});

app.use(express.static("./volvo240.dk"));

console.log("Listening on port", 8002);
app.listen(8002);
