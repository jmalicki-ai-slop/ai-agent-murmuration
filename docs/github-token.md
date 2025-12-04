# GitHub Token Setup

Murmuration requires a GitHub Personal Access Token (PAT) for GitHub API operations.

## Required Permissions

### For Fine-Grained PAT (Recommended)

When creating a fine-grained personal access token at https://github.com/settings/tokens?type=beta:

1. **Repository access**: Select the specific repositories you want Murmuration to access, or "All repositories"

2. **Permissions needed**:

   | Permission | Access Level | Used For |
   |------------|--------------|----------|
   | **Issues** | Read and write | List issues, read issue bodies, create issues from PLAN.md |
   | **Pull requests** | Read | Check PR merge status for dependency resolution |
   | **Metadata** | Read | Repository info (automatically included) |

### For Classic PAT

When creating a classic token at https://github.com/settings/tokens:

Select the `repo` scope, which includes:
- `repo:status` - Access commit status
- `repo_deployment` - Access deployment status
- `public_repo` - Access public repositories
- `repo:invite` - Access repository invitations
- `security_events` - Read security events

The `repo` scope provides broader access than fine-grained tokens but is simpler to configure.

## Setting Up the Token

### Option 1: Environment Variable

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
```

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) for persistence.

### Option 2: File-based (for scripts)

```bash
echo "ghp_xxxxxxxxxxxxxxxxxxxx" > ~/.murmur-gh-token
chmod 600 ~/.murmur-gh-token
export GITHUB_TOKEN=$(cat ~/.murmur-gh-token)
```

## Token Types

| Token Type | Prefix | Works with Murmuration |
|------------|--------|------------------------|
| Fine-grained PAT | `github_pat_` | Yes |
| Classic PAT | `ghp_` | Yes |
| OAuth token (from `gh auth`) | `gho_` | **No** - octocrab requires PAT |

## Verifying Your Token

Test that your token works:

```bash
# Using curl
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user

# Using murmur
murmur issue list --repo owner/repo
```

## Troubleshooting

### "Bad credentials" Error

- Token may be expired or revoked - generate a new one
- Token may not have access to the repository - check repository permissions
- Using OAuth token instead of PAT - create a PAT instead

### "Not Found" Error

- Repository doesn't exist or is private
- Token doesn't have access to this specific repository (for fine-grained tokens)

### Rate Limiting

GitHub API has rate limits:
- Authenticated: 5,000 requests/hour
- Unauthenticated: 60 requests/hour

Murmuration requires authentication for any meaningful use.
