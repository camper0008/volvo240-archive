FROM golang:1.19.2-alpine AS builder
WORKDIR /src
COPY main.go go.mod /src/
RUN ["go", "build", "."]

FROM alpine:3.14.8 AS runner
RUN adduser -D runner
USER runner
WORKDIR /home/runner
COPY --from=builder --chown=runner /src/volvo240_server /home/runner
COPY --chown=runner volvo240.dk /home/runner/volvo240.dk
CMD ["./volvo240_server"]
