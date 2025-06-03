package main

import (
	"dagger/dhd/internal/dagger"
)

func (m *Dhd) arch() *dagger.Container {
	return dag.Container().
		From("archlinux:latest").
		WithExec([]string{"pacman", "-Sy", "--noconfirm", "rust", "nodejs", "npm", "git", "base-devel"})
}
