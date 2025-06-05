import {
    defineModule,
    packageInstall,
    executeCommand,
    userGroup,
} from "@dhd/types";

export default defineModule("docker")
    .description("Container runtime with privilege escalation example")
    .with(() => [
        // Install Docker packages
        packageInstall({
            names: ["docker", "docker-compose"],
        }),
        
        // Add current user to docker group
        userGroup({
            user: "current",
            groups: ["docker"],
            append: true,
        }),
        
        // Disable docker service using privilege escalation
        executeCommand({
            command: "systemctl",
            args: ["disable", "docker.service"],
            privilegeEscalation: true,
        }),
    ]);