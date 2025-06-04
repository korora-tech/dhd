import { defineModule, fileWrite } from "@korora-tech/dhd";

export default defineModule("file")
	.description("Example of fileWrite")
	.with(() => [
		// Basic file creation
		fileWrite({
			destination: "~/.config/myapp/settings.json",
			content: JSON.stringify(
				{
					theme: "dark",
					autoSave: true,
					fontSize: 14,
				},
				null,
				2
			),
		}),

		// Create shell script with executable permissions
		fileWrite({
			destination: "~/bin/backup.sh",
			content: `#!/bin/bash
# Backup script
set -e

echo "Starting backup..."
rsync -av ~/Documents/ ~/backup/Documents/
echo "Backup complete!"
`,
			mode: "755",
		}),

		// Write system configuration file with elevated privileges
		fileWrite({
			destination: "/etc/myapp/config.conf",
			content: `# MyApp Configuration
server.host = 0.0.0.0
server.port = 8080
logging.level = info
`,
			mode: "644",
			privileged: true,
			backup: true,
		}),

		// Create environment file
		fileWrite({
			destination: "~/.env.local",
			content: `# Local environment variables
export API_KEY=your-api-key-here
export DATABASE_URL=postgresql://localhost/mydb
export DEBUG=true
`,
			mode: "600", // Restrict access to owner only
		}),

		// Write HTML template
		fileWrite({
			destination: "~/templates/index.html",
			content: `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My App</title>
</head>
<body>
    <h1>Welcome to My App</h1>
</body>
</html>
`,
		}),
	]);
