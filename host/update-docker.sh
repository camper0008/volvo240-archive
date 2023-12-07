#!/bin/sh
docker stop old-volvo240
docker rm old-volvo240
docker build -t $1 .
docker run -ditp 8081:3333 --name old-volvo240 --restart always -v ./scraper.db:/workspace/scraper.db $1
