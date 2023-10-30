FROM golang:1.21.2-alpine3.18 as builder
WORKDIR /workspace
RUN apk add build-base
COPY main.go .
COPY go.mod .
COPY go.sum .
RUN CGO_ENABLED=1 go build .

FROM alpine:3.18 as runner
WORKDIR /workspace 
COPY assets assets
COPY templates templates
COPY scraper.db .
VOLUME ["/workspace/scraper.db"]
COPY --from=builder /workspace/scraper .
ENTRYPOINT ["/workspace/scraper"]
