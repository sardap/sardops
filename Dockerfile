FROM rust:1.86.0-slim-bookworm

RUN apt-get update -y

RUN mkdir /app
