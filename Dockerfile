FROM rust:1.64

# Rus app compiling
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build
RUN cargo install --path .

# Add docker-compose-wait tool -------------------
ENV WAIT_VERSION 2.9.0
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/$WAIT_VERSION/wait /wait
RUN chmod +x /wait

ENV USER=weeb
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"
USER weeb:weeb

CMD ["animeflv-discord"]