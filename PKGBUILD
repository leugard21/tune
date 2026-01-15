pkgname=tune
pkgver=0.5.0
pkgrel=1
pkgdesc="A terminal-based music player written in Rust"
arch=('x86_64')
url="https://github.com/leugard21/tune"
license=('MIT')
depends=('gcc-libs' 'alsa-lib')
makedepends=('cargo')
source=()
sha256sums=()

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    if [ -f Cargo.lock ]; then
        cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
    fi
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --release --locked --all-features
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo test --frozen --all-features
}

package() {
    install -Dm 755 "target/release/tune" "$pkgdir/usr/bin/tune"
    install -Dm 644 "$startdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm 644 "$startdir/README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
