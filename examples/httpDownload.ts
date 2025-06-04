import { defineModule, httpDownload } from "@korora-tech/dhd";

export default defineModule("httpDownload")
	.description("Example of httpDownload")
	.with(() => [
		// Basic file download
		httpDownload({
			url: "https://github.com/user/repo/releases/download/v1.0.0/tool-linux-amd64",
			destination: "~/bin/tool",
		}),

		// Download with checksum verification
		httpDownload({
			url: "https://github.com/user/repo/releases/download/v1.0.0/tool-linux-amd64",
			destination: "~/bin/tool",
			checksum: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
			checksumType: "sha256",
			mode: "755",
		}),

		// Download system binary with elevated privileges
		httpDownload({
			url: "https://github.com/user/repo/releases/download/v1.0.0/tool-linux-amd64",
			destination: "/usr/local/bin/tool",
			checksum: "d41d8cd98f00b204e9800998ecf8427e",
			checksumType: "md5",
			mode: "755",
			privileged: true,
		}),

		// Download configuration file
		httpDownload({
			url: "https://raw.githubusercontent.com/user/repo/main/config/default.conf",
			destination: "/etc/myapp/default.conf",
			checksum: "da39a3ee5e6b4b0d3255bfef95601890afd80709",
			checksumType: "sha512",
			mode: "644",
			privileged: true,
		}),
	]);
