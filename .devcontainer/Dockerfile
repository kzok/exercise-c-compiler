# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.140.1/containers/ubuntu/.devcontainer/base.Dockerfile

# [Choice] Ubuntu version: bionic, focal
FROM mcr.microsoft.com/vscode/devcontainers/base:0-focal

SHELL ["/bin/bash", "-c"]
ENV LC_ALL C.UTF-8

RUN apt -qq -y update && apt -qq -y install git build-essential

# Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
RUN source $HOME/.cargo/env \
  && rustup update \
  && rustup component add rls rust-analysis rust-src

# /etc/bash.bashrc
RUN touch /etc/bash.bashrc \
  && echo 'umask 0000' >> /etc/bash.bashrc
