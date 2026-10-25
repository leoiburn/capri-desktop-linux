FROM fedora:44

# Install basic X11 and Wayland client libraries needed for desktop GUI execution
RUN dnf install -y \
    libX11 \
    libXcursor \
    libxkbcommon \
    xorg-x11-server-Xvfb \
    mesa-dri-drivers \
    curl \
    ca-certificates \
    && dnf clean all

# Copy and install the built Capri RPM package
COPY capri-desktop-1.0.0-1.fc44.x86_64.rpm /tmp/
RUN dnf install -y /tmp/capri-desktop-1.0.0-1.fc44.x86_64.rpm && rm /tmp/capri-desktop-1.0.0-1.fc44.x86_64.rpm

# Verify files are properly placed in standard Linux filesystem
RUN test -x /usr/bin/capri && \
    test -L /usr/bin/capri-desktop && \
    test -f /usr/share/applications/capri-desktop.desktop && \
    test -f /usr/share/icons/hicolor/scalable/apps/capri-desktop.png

# Entrypoint test script
COPY test_loop.sh /root/test_loop.sh
RUN chmod +x /root/test_loop.sh

ENTRYPOINT ["/root/test_loop.sh"]
