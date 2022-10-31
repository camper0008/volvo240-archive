# frontend

FROM node:16

COPY . /docker

WORKDIR /docker

RUN npm ci --only=production
