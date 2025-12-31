<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |
| < main  | :x:                |

## Reporting a Vulnerability

Please report security vulnerabilities through GitHub private vulnerability reporting:
1. Go to the **Security** tab
2. Click **Report a vulnerability**
3. Fill out the form

We respond within 48 hours.

## Security Measures

- Dependabot for dependency updates
- CodeQL for code scanning
- Secret scanning and push protection
- Secure API key handling with memory zeroization
- HTTPS-only network requests
- Input validation at system boundaries

## API Key Security

API keys are handled securely:

- Stored using `SecureApiKey` wrapper with `zeroize` crate
- Memory is cleared on drop
- Never shown in debug output (`[REDACTED]`)
- Environment variable auto-detection

## Security Best Practices

1. Store API keys in environment variables, not in code
2. Regularly update dependencies
3. Do not run untrusted `.my` files
4. Use HTTPS for all external connections

