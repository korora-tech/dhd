# @dhd/types

TypeScript type definitions for DHD (Declarative Home Deployments).

## Installation

```bash
# Using Bun
bunx jsr add @dhd/types

# Using npm
npx jsr add @dhd/types

# Using Deno
import { defineModule, packageInstall } from "jsr:@dhd/types";
```

## Usage

```typescript
import { defineModule, packageInstall, executeCommand } from "@dhd/types";

export default defineModule("my-module")
  .description("My custom module")
  .with((ctx) => [
    packageInstall({ names: ["neovim"] }),
    executeCommand({ command: "echo", args: ["Hello, DHD!"] })
  ]);
```

## License

MIT