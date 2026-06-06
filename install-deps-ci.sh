#!/usr/bin/env bash

apt update

# Tauri dependencies
apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# swipl runtime dependencies
apt install libgoogle-perftools-dev
