# Build stage
FROM rust:1.75-alpine3.19 as builder

RUN apk add musl-dev
# Copy the source code
ADD . .

# Build the project
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release

# Final stage
FROM alpine:3.19

ARG REF=""
ARG COMMIT=""
ARG TIME=""

ENV COMMIT=${COMMIT}
ENV REF=${REF}
ENV TIME=${TIME}
ENV CT=${CT}
ENV TZ="America/New_York"

# Copy the binary from the build stage
COPY --from=builder /target/release/umaring /usr/local/bin/umaring
COPY ./js /usr/local/bin/js

# Set the command to run the binary
WORKDIR /usr/local/bin
CMD ["umaring"]
EXPOSE 3000
