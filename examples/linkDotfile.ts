import { defineModule, linkDotfile } from "../types";

export default defineModule("linkDotfile")
  .description("Example of linkDotfile")
  .with(() => [
    // Example of providing a relative target
    // Relative targets are relative to XDG_CONFIG_HOME
    linkDotfile({
      source: "zellij.kdl",
      target: "zellij/config.kdl",
    }),

    // Example of source and target being the same
    linkDotfile({
      source: "zellij.kdl",
    }),
  ]);
