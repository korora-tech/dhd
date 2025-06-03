package main

import (
	"dagger/dhd/internal/dagger"
)

func (m *Dhd) debian() *dagger.Container {
	return dag.Container().
		From("debian:latest").
		WithExec([]string{"apt-get", "update"}).
		WithExec([]string{"apt-get", "install", "-y", "curl", "git", "build-essential"}).
		// Install Rust
		WithExec([]string{"sh", "-c", "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"}).
		WithEnvVariable("PATH", "/root/.cargo/bin:$PATH", dagger.ContainerWithEnvVariableOpts{Expand: true}).
		// Install Node.js
		WithExec([]string{"sh", "-c", "curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -"}).
		WithExec([]string{"apt-get", "install", "-y", "nodejs"})
}