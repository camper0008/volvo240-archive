#!/bin/sh

# extract posts as json
sqlite3 scraper.db < ".mode json\n.once posts.json\n.SELECT * FROM post;"

# create post index
curl \
  -X POST 'http://localhost:7700/indexes' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "post"
  }'

# add documents
curl \
  -X POST 'http://localhost:7700/indexes/post/documents' \
  -H 'Content-Type: application/json' \
  --data-binary "$(cat posts.json)"
