async function search(query, page, limit, key) {
    // example response:
    // {
    //   "hits": [
    //     {
    //       "forum_id": 25684,
    //       "post_id": 25684,
    //       "title": "Hjælp søges!",
    //       "content": "Hvordan skifter jeg til vinterdæk?",
    //       "author": "Pieter",
    //       "_formatted": {
    //           "forum_id": 25684,
    //           "post_id": 25684,
    //           "title": "Hjælp søges!",
    //           "content": "Hvordan skifter jeg til <span class="highlighted">vinter</span>dæk?",
    //           "author": "Pieter",
    //       },
    //     }
    //   ],
    //   "query": "vinter",
    //   "totalPages": 0
    // }
    const index = "post";
    const api_url = "https://meili.volvo240.dk"
    const url = `${api_url}/indexes/${index}/search?q=${query}&attributesToRetrieve=title,content,author,post_id,date,forum_id&attributesToHighlight=title,content&highlightPreTag=<span class="highlighted">&highlightPostTag=</span>&page=${page}&hitsPerPage=${limit}`;
    const headers = new Headers({"Authorization": `Bearer ${key}`});

    return await (await fetch(url, { headers })).json();
}

let page = 1;
let total_pages = 1;

function update_page_buttons() {
    const page_counter = document.getElementById("page-counter");
    page_counter.innerText = (page - 1).toString();

    const page_switcher = document.getElementById("page-switcher");
    if (total_pages <= 1) {
        page_switcher.setAttribute("style", "display: none;");
        return;
    } else {
        page_switcher.removeAttribute("style");
    }
    const next_page_anchor = document.getElementById("next-page");
    if (page < total_pages) {
        next_page_anchor.setAttribute("href", "#");
    } else {
        next_page_anchor.removeAttribute("href");
    }

    const prev_page_anchor = document.getElementById("prev-page");
    if (page > 1) {
        prev_page_anchor.setAttribute("href", "#");
    } else {
        prev_page_anchor.removeAttribute("href");
    }
}

function next_page(event, key) {
    event.preventDefault();
    const search = document.getElementById("search");
    if (page < total_pages) {
        page += 1;
        on_input(search.value, key);
        update_page_buttons();
    }
}

function previous_page(event, key) {
    event.preventDefault();
    const search = document.getElementById("search");
    if (page > 1) {
        page -= 1;
        on_input(search.value, key);
        update_page_buttons();
    }
}

async function on_input(query, key) {
    const results = document.getElementById("search-results");
    const search_info = document.getElementById("search-info");

    if (query.trim() === "") {
        results.innerHTML = "<p class='center'>Indtast noget i søgefeltet for at begynde</p>"
        search_info.setAttribute("style", "display: none;");
        total_pages = 0;
        update_page_buttons();
        return;
    }

    const response = await search(query, page, 5, key);
    total_pages = response.totalPages;
    const new_results = response.hits
        .map((hit) => {
            const card = document.createElement("div");
            card.classList.add("card");
            const title = document.createElement("p");
            title.innerHTML = hit._formatted.title
            title.innerHTML = `<b>${title.innerHTML}</b> <span class='author'>af ${hit.author} d. ${hit.date}</span>`;
            const content = document.createElement("p");
            content.classList.add("text-post");
            content.innerHTML = hit._formatted.content;
            const link = document.createElement("a");
            const back = query.trim() !== "" ? encodeURIComponent(`/search?q=${query}`) : "/search"
            link.href = `/post?post=${hit.post_id}&forum=${hit.forum_id}&back=${back}`
            link.innerText = "Se opslag";
            card.append(title, content, link);
            return card.outerHTML;
        })
        .join("<hr>");
    
    search_info.innerText = response.totalPages > 1 
        ? `Fandt ${response.totalHits} resultater fordelt over ${response.totalPages} sider.` 
        : `Fandt ${response.totalHits} resultater.`;

    const next_page_anchor = document.getElementById("next-page");
    const prev_page_anchor = document.getElementById("prev-page");

    if (query.trim() !== "") {
        results.innerHTML = new_results;
        search_info.removeAttribute("style");
        update_page_buttons();
    }
}

async function main() {
    const key = await (await fetch("https://meili.volvo240.dk/public/search_key")).text();
    const search = document.getElementById("search");
    search.addEventListener("input", () => {page = 1; on_input(search.value, key);});
    on_input(search.value, key);

    const next_page_anchor = document.getElementById("next-page");
    next_page_anchor.addEventListener("click", (event) => next_page(event, key));
    const prev_page_anchor = document.getElementById("prev-page");
    prev_page_anchor.addEventListener("click", (event) => previous_page(event, key));
    update_page_buttons();
}

main();
