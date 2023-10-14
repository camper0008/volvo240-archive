FROM golang:1.21.2 as builder
WORKDIR /workspace
COPY main.go .
COPY go.mod .
COPY go.sum .
RUN go build .

FROM golang:1.21.2 as runner
WORKDIR /workspace 
COPY scraper.db .
COPY assets assets
COPY templates templates
COPY --from=builder /workspace/scraper .
ENTRYPOINT ["/workspace/scraper"]
