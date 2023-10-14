#!/bin/sh
docker run -ditp 8081:3333 --name old-volvo240 --restart always $1
