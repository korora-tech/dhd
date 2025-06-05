import { defineModule, packageInstall, userGroup } from "@dhd/types";

export default defineModule("docker")
    .description("Container runtime with Docker and Docker Compose")
    .with(() => [
        // Install Docker packages
        packageInstall({
            names: ["docker", "docker-compose"],
        }),
        
        // Add current user to docker group
        // Special values: "current" or "${USER}" will use the current user
        userGroup({
            user: "current",
            groups: ["docker"],
            append: true,
        }),
    ]);