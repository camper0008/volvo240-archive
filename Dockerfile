FROM golang:1.19.2-alpine AS builder
WORKDIR /src
COPY main.go go.mod /src/
# statically link go binary/no external dependencies, allows running in scratch docker image
ENV CGO_ENABLED=0
RUN ["go", "build", "."]

FROM scratch
COPY --from=builder /src/volvo240_server /
COPY volvo240.dk /volvo240.dk
EXPOSE 8080
CMD ["./volvo240_server"]
