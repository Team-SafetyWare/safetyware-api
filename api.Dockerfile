FROM rust:1.56.1

WORKDIR /usr/src/api
COPY api .

RUN cargo install --path .

CMD ["api"]
