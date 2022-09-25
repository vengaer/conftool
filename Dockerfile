FROM archlinux:latest
LABEL maintainer="vilhelm.engstrom@tuta.io"

COPY . /conftool
WORKDIR /conftool

RUN useradd -m builder                      &&  \
    pacman -Syu --noconfirm --needed rust   && \
    chown -R builder:builder /conftool

USER builder
