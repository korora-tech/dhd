import { defineModule, packageInstall } from "@dhd/types";

// NOTE: This example shows the INTENDED API, but it doesn't actually work
// because DHD uses static parsing, not runtime execution of TypeScript

export default defineModule("platform-example")
    .description("Example of platform selection (non-functional)")
    .with((ctx) => [
        // This WOULD work if TypeScript was executed:
        packageInstall({
            names: ctx.platform.select({
                default: ["curl"],
                mac: ["curl", "wget"],
                windows: ["curl-win64"],
                linux: {
                    distro: {
                        ubuntu: ["curl", "wget"],
                        arch: ["curl", "wget", "aria2"],
                    }
                }
            }),
        }),
        
        // But in reality, you must use static values:
        // packageInstall({
        //     names: ["curl", "wget"],
        // }),
    ]);