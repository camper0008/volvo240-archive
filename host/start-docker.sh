#!/bin/sh
docker run -ditp 8081:3333 --name old-volvo240 --restart always -v scraper.db:/workspace/scraper.db $1
