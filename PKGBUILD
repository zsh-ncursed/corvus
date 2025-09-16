# Maintainer: Jules <jules@example.com>
pkgname=corvus
pkgver=0.1.0
pkgrel=1
pkgdesc="A fast, extensible, and cross-platform terminal file manager with image previews, built in Rust."
arch=('x86_64')
url="https://github.com/user/corvus" # Placeholder URL
license=('MIT')
depends=('mupdf')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP') # In a real scenario, this would be the checksum of the source tarball

prepare() {
    cd "$pkgname-$pkgver"
    rustup default stable
}

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

check() {
    cd "$pkgname-$pkgver"
    cargo test --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/corvus" "$pkgdir/usr/bin/corvus"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
