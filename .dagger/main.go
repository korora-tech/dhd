// DHD (Declarative Home Deployments) Dagger module
//
// This module provides functions to test and run DHD examples across different
// Linux distributions. It helps ensure DHD works correctly on various platforms
// by running the examples in containerized environments.

package main

import (
	"context"
	"dagger/dhd/internal/dagger"
)

type Dhd struct{}

// Build compiles the DHD binary from source
func (m *Dhd) Build(ctx context.Context, source *dagger.Directory) *dagger.Container {
	return dag.Container().
		From("rust:latest").
		WithMountedDirectory("/src", source).
		WithWorkdir("/src").
		WithMountedCache("/usr/local/cargo/registry", dag.CacheVolume("cargo-registry")).
		WithMountedCache("/src/target", dag.CacheVolume("rust-target")).
		WithExec([]string{"cargo", "build", "--release"})
}

// RunExamples runs the DHD examples in both Arch and Debian containers
func (m *Dhd) RunExamples(ctx context.Context, source *dagger.Directory) error {
	// Build the binary once
	buildContainer := m.Build(ctx, source)
	binary := buildContainer.File("/src/target/release/dhd")

	// Run in Arch Linux
	archContainer := m.arch().
		WithMountedDirectory("/app", source).
		WithWorkdir("/app").
		WithFile("/app/dhd", binary)
	archResult, archErr := m.runExample(ctx, archContainer)

	// Run in Debian
	debianContainer := m.debian().
		WithMountedDirectory("/app", source).
		WithWorkdir("/app").
		WithFile("/app/dhd", binary)
	debianResult, debianErr := m.runExample(ctx, debianContainer)

	// Print results
	if archErr != nil {
		println("Arch Linux run failed:", archErr.Error())
	} else {
		println("Arch Linux output:", archResult)
	}

	if debianErr != nil {
		println("Debian run failed:", debianErr.Error())
	} else {
		println("Debian output:", debianResult)
	}

	// Return success even if examples fail (as requested)
	return nil
}

// Helper function to run the example with pre-built binary
func (m *Dhd) runExample(ctx context.Context, container *dagger.Container) (string, error) {
	return container.
		WithExec([]string{"bun", "install"}).
		WithExec([]string{"/app/dhd", "apply", "--modules", "packageInstall", "--modules-path", "examples"}).
		Stdout(ctx)
}
