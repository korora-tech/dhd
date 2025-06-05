import {
    defineModule,
    userGroup,
} from "@dhd/types";

export default defineModule("docker-user")
    .description("Example of using userGroup")
    .with(() => [
        // The special value "current" is automatically resolved to the actual username
        // in the Rust implementation
        userGroup({
            user: "current",
            groups: ["docker", "libvirt"],
            append: true,
        }),
        
        // You can also specify a specific username
        userGroup({
            user: "alice",
            groups: ["sudo", "admin"],
            append: true,
        }),
    ]);