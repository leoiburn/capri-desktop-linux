#!/usr/bin/env bash
set -euo pipefail

[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
export PATH="$HOME/.cargo/bin:$PATH"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

echo "=== Building Capri Desktop Release Binary ==="
cargo build --release --manifest-path capri-rust/Cargo.toml

echo "=== Preparing RPM Sources ==="
mkdir -p rpmbuild/SOURCES rpmbuild/RPMS rpmbuild/SRPMS rpmbuild/BUILD rpmbuild/BUILDROOT
tar -czvf rpmbuild/SOURCES/capri-desktop-1.0.0.tar.gz -C capri-rust/target/release capri

echo "=== Building RPM Package ==="
rpmbuild --define "_topdir ${SCRIPT_DIR}/rpmbuild" -ba rpmbuild/SPECS/capri-desktop.spec

RPM_PATH="$(find rpmbuild/RPMS/ -name "capri-desktop-*.rpm" | head -n 1)"
if [ -n "${RPM_PATH}" ]; then
    cp "${RPM_PATH}" "${SCRIPT_DIR}/capri-desktop-1.0.0-1.fc44.x86_64.rpm"
    echo "=== RPM Build Completed Successfully ==="
    echo "Output: ${RPM_PATH}"
fi
