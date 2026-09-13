#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install --no-install-recommends -y \
  pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libxdo-dev \
  libayatana-appindicator3-dev libwayland-dev libxkbcommon-dev \
  libegl1-mesa-dev libgl1-mesa-dri mesa-vulkan-drivers \
  mutter dbus-x11 xwayland fuse3 python3-xlib
