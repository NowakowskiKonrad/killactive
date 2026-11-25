pkgname=killactive-git
pkgver=0.1.0.r0.g0000000
pkgrel=1
pkgdesc="Instant SUPER+Q replacement for ML4W killactive.sh"
arch=('x86_64' 'aarch64')
url="https://github.com/NowakowskiKonrad/killactive"
license=('MIT')
depends=('gcc-libs' 'glibc')
makedepends=('cargo' 'git')
provides=('killactive')
conflicts=('killactive')
optdepends=(
    'hyprland: target compositor'
    'quickshell: target overlay panel daemon'
)
options=('!lto')
source=("git+$url.git")
sha256sums=('SKIP')

pkgver() {
    cd "${pkgname%-git}"
    printf "0.1.0.r%s.g%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short=7 HEAD)"
}

prepare() {
    cd "${pkgname%-git}"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "${pkgname%-git}"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release
}

check() {
    cd "${pkgname%-git}"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen
}

package() {
    cd "${pkgname%-git}"
    install -Dm0755 "target/release/${pkgname%-git}" "$pkgdir/usr/bin/${pkgname%-git}"
    install -Dm0644 README.md "$pkgdir/usr/share/doc/${pkgname%-git}/README.md"
    install -Dm0644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}