#!/usr/bin/env bash

apt-add-repository ppa:swi-prolog/stable
apt update
apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  swi-prolog=10.0.2-*
