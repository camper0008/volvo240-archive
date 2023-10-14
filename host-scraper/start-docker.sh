#!/bin/sh
docker run -ditp 8081:3333 --restart always $1
