/**
 * Example module demonstrating 1Password integration with dhd
 * 
 * This module shows how to use 1Password secrets in various actions
 * and conditions.
 */

export default defineModule("1password-example")
  .description("Example of using 1Password secrets in dhd")
  .tags(["secrets", "1password", "example"])
  .actions([
    // Example 1: Using secrets in environment variables for commands
    executeCommand({
      command: "git",
      args: ["clone", "https://github.com/private/repo.git"],
      environment: {
        // Reference a GitHub token stored in 1Password
        GIT_TOKEN: "op://Personal/GitHub/token",
        // You can also use environment variables as fallback
        API_KEY: "env://API_KEY",
        // Or literal values for non-sensitive data
        APP_ENV: "literal://production"
      }
    }),

    // Example 2: Deploy a service with database credentials from 1Password
    executeCommand({
      command: "docker",
      args: ["run", "-d", "--name", "myapp", "myapp:latest"],
      environment: {
        DATABASE_URL: "op://Work/Production DB/connection_string",
        REDIS_PASSWORD: "op://Work/Redis/password",
        JWT_SECRET: "op://Work/API/jwt_secret"
      }
    }),

    // Example 3: Configure AWS CLI with credentials from 1Password
    executeCommand({
      command: "aws",
      args: ["configure", "set", "aws_access_key_id"],
      environment: {
        AWS_ACCESS_KEY_ID: "op://Work/AWS/access_key_id"
      }
    }),

    // Example 4: Use conditional execution based on secret availability
    onlyIf(
      secretExists("op://Personal/SSH Keys/github")
    ).then(
      executeCommand({
        command: "ssh-add",
        args: ["-"],
        environment: {
          SSH_PRIVATE_KEY: "op://Personal/SSH Keys/github"
        }
      })
    ),

    // Example 5: Set up Kubernetes config with service account token
    executeCommand({
      command: "kubectl",
      args: ["config", "set-credentials", "prod-user"],
      environment: {
        K8S_TOKEN: "op://Work/Kubernetes/prod_token"
      }
    }),

    // Example 6: Install and configure a service with multiple secrets
    packageInstall({
      names: ["postgresql"],
      manager: "apt"
    }),

    executeCommand({
      command: "sudo",
      args: ["-u", "postgres", "psql"],
      environment: {
        PGPASSWORD: "op://Work/PostgreSQL/admin_password"
      }
    }),

    // Example 7: Create a systemd service with secrets
    systemdService({
      unit: "myapp",
      content: `[Unit]
Description=My Application
After=network.target

[Service]
Type=simple
User=myapp
WorkingDirectory=/opt/myapp
ExecStart=/opt/myapp/bin/start
Restart=always
EnvironmentFile=/etc/myapp/env

[Install]
WantedBy=multi-user.target`
    }),

    // Write environment file with secrets
    executeCommand({
      command: "bash",
      args: ["-c", "echo 'DATABASE_URL=$DATABASE_URL' > /etc/myapp/env"],
      environment: {
        DATABASE_URL: "op://Work/MyApp/database_url"
      },
      escalate: true
    }),

    // Example 8: Backup with encryption key from 1Password
    executeCommand({
      command: "restic",
      args: ["backup", "/important/data"],
      environment: {
        RESTIC_REPOSITORY: "s3:s3.amazonaws.com/mybucket",
        RESTIC_PASSWORD: "op://Personal/Backup/encryption_key",
        AWS_ACCESS_KEY_ID: "op://Personal/AWS/access_key",
        AWS_SECRET_ACCESS_KEY: "op://Personal/AWS/secret_key"
      }
    }),

    // Example 9: Deploy with Docker Compose using secrets
    executeCommand({
      command: "docker-compose",
      args: ["up", "-d"],
      environment: {
        POSTGRES_PASSWORD: "op://Work/Database/postgres_password",
        REDIS_PASSWORD: "op://Work/Redis/password",
        APP_SECRET_KEY: "op://Work/App/secret_key",
        SMTP_PASSWORD: "op://Work/Email/smtp_password"
      }
    }),

    // Example 10: Certificate management
    executeCommand({
      command: "certbot",
      args: ["certonly", "--dns-cloudflare", "-d", "*.example.com"],
      environment: {
        CLOUDFLARE_API_TOKEN: "op://Work/Cloudflare/api_token"
      }
    })
  ])
  .when(
    // Only run this module if 1Password CLI is available
    commandExists("op")
  );