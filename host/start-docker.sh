#!/bin/sh
read -p "username: " USERNAME
read -p "Password: " PASSWORD
docker run -ditp 8081:3333 --name old-volvo240 -e "USERNAME=$USERNAME" -e "PASSWORD=$PASSWORD" --restart always -v ./scraper.db:/workspace/scraper.db $1
