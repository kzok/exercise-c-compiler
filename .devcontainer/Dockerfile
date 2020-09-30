# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.140.1/containers/ubuntu/.devcontainer/base.Dockerfile

# [Choice] Ubuntu version: bionic, focal
ARG VARIANT="focal"
FROM mcr.microsoft.com/vscode/devcontainers/base:0-${VARIANT}

ENV LC_ALL C.UTF-8

RUN apt -qq -y update && apt -qq -y install git build-essential

# Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
RUN rustup update
RUN rustup component add rls rust-analysis rust-src
