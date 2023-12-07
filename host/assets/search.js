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

async function onInput(query, key) {
    const response = await search(query, 1, 5, key);
    const results = document.getElementById("search-results");
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
    results.innerHTML = new_results;
}

async function main() {
    const key = await (await fetch("https://meili.volvo240.dk/public/search_key")).text();
    const search = document.getElementById("search");
    search.addEventListener("input", () => onInput(search.value, key));
    onInput(search.value, key);
}

main();
