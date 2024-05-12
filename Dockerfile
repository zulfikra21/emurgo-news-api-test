FROM rust:1.77-slim-bookworm


WORKDIR /home/app

COPY . .
COPY .env .

RUN apt-get update && apt-get install -y
RUN apt-get upgrade -y
RUN apt install pkg-config -y
RUN apt-get install libudev-dev -y
RUN apt-get install libssl-dev -y
RUN apt install libpq-dev -y
RUN apt-get upgrade openssl


# RUN apt-get install libressl-dev -y
