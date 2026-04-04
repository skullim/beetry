FROM rust:1.94-trixie

SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN apt-get update && \
    apt-get install -y \
    build-essential  \
    sudo \
    bash-completion \
    curl \
    wget \
    file \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    locales && \
    printf 'en_US.UTF-8 UTF-8\n' > /etc/locale.gen && \
    locale-gen

RUN rustup component add clippy rustfmt && \
    rustup install nightly-x86_64-unknown-linux-gnu --component rustfmt  &&\
    cargo install cargo-binstall &&\
    cargo binstall --disable-telemetry -y dioxus-cli

ARG USERNAME=dev
ARG USER_UID=1000
ARG USER_GID=1000

RUN groupadd --gid ${USER_GID} ${USERNAME} \
    && useradd --uid ${USER_UID} --gid ${USER_GID} --create-home --shell /bin/bash ${USERNAME} \
    && echo "dev ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/${USERNAME} \
    && chmod 0440 /etc/sudoers.d/${USERNAME} \
    && chown -R ${USER_UID}:${USER_GID} /usr/local/cargo /usr/local/rustup

USER ${USERNAME}
WORKDIR /home/${USERNAME}/
