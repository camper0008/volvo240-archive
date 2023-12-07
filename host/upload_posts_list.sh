#!/bin/sh

## upload posts to meilisearch

set -xe

# extract posts as json
sqlite3 scraper.db << EOF
.mode json
.once posts.json
SELECT rowid as id, * FROM post;
EOF

read -p "authorization key: "

# create post index
curl \
  -X POST 'http://localhost:7700/indexes' \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $REPLY" \
  --data-binary '{
    "uid": "post"
  }'

# add documents
curl \
  -X POST 'http://localhost:7700/indexes/post/documents?primaryKey=id' \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $REPLY" \
  --data-binary "@posts.json"

