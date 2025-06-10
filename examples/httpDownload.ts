export default defineModule("httpDownload")
    .description("Download files from HTTP URLs with checksum verification")
    .actions([
        httpdownload({
            url: "https://github.com/BurntSushi/ripgrep/releases/download/14.1.0/ripgrep-14.1.0-x86_64-unknown-linux-musl.tar.gz",
            destination: "~/.local/bin/ripgrep.tar.gz",
            checksum: {
                algorithm: "sha256",
                value: "35dc6726cd84d296def5e820b76f38b9ac2bf90e0e5f19b53d13eb9ee7b0b9b1",
            },
        }),
        httpdownload({
            url: "https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh",
            destination: "~/.local/bin/install-ohmyzsh.sh",
            mode: 0o755,
        }),
        httpdownload({
            url: "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-unknown-linux-musl.tar.gz",
            destination: "/tmp/bat.tar.gz",
            checksum: {
                algorithm: "sha256",
                value: "907554a9eff239f256ee8fe05a922aad84febe4fe10a499def72a4557e9eedfb",
            },
            mode: 0o644,
        }),
    ]);
