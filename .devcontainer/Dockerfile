FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

USER vscode

RUN sudo apt-get update

RUN curl -L https://foundry.paradigm.xyz | bash

SHELL ["/bin/bash", "--login", "-i", "-c"]
RUN foundryup