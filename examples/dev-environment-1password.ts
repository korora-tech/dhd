/**
 * Development Environment Setup with 1Password Secrets
 * 
 * This module sets up a complete development environment with credentials
 * and API keys stored securely in 1Password.
 */

export default defineModule("dev-environment")
  .description("Set up development environment with 1Password secrets")
  .tags(["development", "setup", "1password"])
  .when(
    allOf([
      commandExists("op"),
      secretExists("op://Personal/GitHub/token")
    ])
  )
  .actions([
    // Configure Git with signing key from 1Password
    gitConfig({
      scope: "global",
      settings: {
        "user.name": "Your Name",
        "user.email": "your.email@example.com",
        "commit.gpgsign": "true",
        "user.signingkey": "YOUR_GPG_KEY_ID"
      }
    }),

    // Import GPG key from 1Password
    executeCommand({
      command: "gpg",
      args: ["--import"],
      environment: {
        GPG_PRIVATE_KEY: "op://Personal/GPG/private_key"
      }
    }),

    // Configure GitHub CLI with token
    executeCommand({
      command: "gh",
      args: ["auth", "login", "--with-token"],
      environment: {
        GITHUB_TOKEN: "op://Personal/GitHub/token"
      }
    }),

    // Set up SSH keys
    directory({
      path: "~/.ssh",
      mode: "0700"
    }),

    executeCommand({
      command: "bash",
      args: ["-c", "echo '$SSH_KEY' > ~/.ssh/id_ed25519 && chmod 600 ~/.ssh/id_ed25519"],
      environment: {
        SSH_KEY: "op://Personal/SSH/github_private_key"
      }
    }),

    executeCommand({
      command: "bash",
      args: ["-c", "echo '$SSH_PUB_KEY' > ~/.ssh/id_ed25519.pub && chmod 644 ~/.ssh/id_ed25519.pub"],
      environment: {
        SSH_PUB_KEY: "op://Personal/SSH/github_public_key"
      }
    }),

    // Configure npm with auth token
    executeCommand({
      command: "npm",
      args: ["config", "set", "//registry.npmjs.org/:_authToken", "${NPM_TOKEN}"],
      environment: {
        NPM_TOKEN: "op://Work/NPM/auth_token"
      }
    }),

    // Set up Docker Hub credentials
    executeCommand({
      command: "docker",
      args: ["login", "-u", "${DOCKER_USER}", "-p", "${DOCKER_PASS}"],
      environment: {
        DOCKER_USER: "op://Personal/Docker Hub/username",
        DOCKER_PASS: "op://Personal/Docker Hub/password"
      }
    }),

    // Configure AWS CLI
    executeCommand({
      command: "aws",
      args: ["configure", "set", "aws_access_key_id", "${AWS_KEY_ID}"],
      environment: {
        AWS_KEY_ID: "op://Work/AWS/access_key_id"
      }
    }),

    executeCommand({
      command: "aws",
      args: ["configure", "set", "aws_secret_access_key", "${AWS_SECRET}"],
      environment: {
        AWS_SECRET: "op://Work/AWS/secret_access_key"
      }
    }),

    // Set up Kubernetes contexts
    executeCommand({
      command: "kubectl",
      args: ["config", "set-cluster", "prod", "--server=https://k8s.example.com"],
    }),

    executeCommand({
      command: "kubectl",
      args: ["config", "set-credentials", "prod-admin", "--token=${K8S_TOKEN}"],
      environment: {
        K8S_TOKEN: "op://Work/Kubernetes/prod_admin_token"
      }
    }),

    // Create development environment file
    executeCommand({
      command: "bash",
      args: ["-c", `cat > ~/.env.development << EOF
export GITHUB_TOKEN=\$(op read "op://Personal/GitHub/token")
export NPM_TOKEN=\$(op read "op://Work/NPM/auth_token")
export AWS_ACCESS_KEY_ID=\$(op read "op://Work/AWS/access_key_id")
export AWS_SECRET_ACCESS_KEY=\$(op read "op://Work/AWS/secret_access_key")
export DOCKER_HOST=\${DOCKER_HOST:-unix:///var/run/docker.sock}
EOF`]
    }),

    // Add sourcing to shell profile
    executeCommand({
      command: "bash",
      args: ["-c", "echo 'source ~/.env.development' >> ~/.bashrc"]
    }),

    // Set up database connection aliases
    executeCommand({
      command: "bash",
      args: ["-c", `cat > ~/.pgpass << EOF
localhost:5432:myapp:myuser:\$(op read "op://Work/PostgreSQL/dev_password")
prod.db.example.com:5432:myapp:myuser:\$(op read "op://Work/PostgreSQL/prod_password")
EOF && chmod 600 ~/.pgpass`]
    }),

    // Configure HashiCorp Vault
    executeCommand({
      command: "vault",
      args: ["login", "-method=token"],
      environment: {
        VAULT_TOKEN: "op://Work/Vault/root_token",
        VAULT_ADDR: "https://vault.example.com"
      }
    }),

    // Set up Terraform with cloud credentials
    executeCommand({
      command: "terraform",
      args: ["login"],
      environment: {
        TF_TOKEN: "op://Work/Terraform Cloud/api_token"
      }
    }),

    // Configure Ansible vault
    executeCommand({
      command: "bash",
      args: ["-c", "echo '$ANSIBLE_VAULT_PASS' > ~/.ansible_vault_pass && chmod 600 ~/.ansible_vault_pass"],
      environment: {
        ANSIBLE_VAULT_PASS: "op://Work/Ansible/vault_password"
      }
    }),

    // Set up monitoring and observability tools
    executeCommand({
      command: "bash",
      args: ["-c", `cat > ~/.config/datadog.yaml << EOF
api_key: \$(op read "op://Work/Datadog/api_key")
app_key: \$(op read "op://Work/Datadog/app_key")
EOF`]
    })
  ]);