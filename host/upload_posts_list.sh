#!/bin/sh

## upload posts to meilisearch

# extract posts as json
sqlite3 scraper.db < ".mode json\n.once posts.json\n.SELECT * FROM post;"

read -p "authorization key: "

# create post index
curl \
  -X POST 'http://localhost:7700/indexes' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer $REPLY' \
  --data-binary '{
    "uid": "post"
  }'

# add documents
curl \
  -X POST 'http://localhost:7700/indexes/post/documents' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer $REPLY' \
  --data-binary "$(cat posts.json)"
