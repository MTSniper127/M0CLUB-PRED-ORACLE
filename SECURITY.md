
# Security Policy

M0Club takes security seriously. This document explains how to report vulnerabilities,
how we handle disclosures, and the security expectations for contributors.

## Supported Versions

We provide security updates for the following versions:

| Version | Supported |
|--------:|:---------:|
| 0.1.x   | ✅        |
| < 0.1.0 | ❌        |

## Reporting a Vulnerability

### Do not disclose publicly

Please **do not** open a public issue or post details publicly until we have reviewed and responded.

### Contact

Send an email to:
- security@m0club.com

If you believe the issue is actively being exploited, also contact:
- conduct@m0club.com

### What to include

Include as much of the following as possible:

- A clear description of the issue and its impact
- Affected components (programs / engine / services / sdk / infra)
- Steps to reproduce
- Proof-of-concept code or payloads (if available)
- Logs, stack traces, or error output
- Any suggested remediation

### Expected response timeline

- Acknowledgement: within **72 hours**
- Initial triage: within **7 days**
- Fix or mitigation: depends on severity and complexity

## Disclosure Process

We follow a coordinated disclosure process:

1. **Receipt**: We confirm receipt and assign a private tracking ID.
2. **Triage**: We reproduce the issue and assess severity.
3. **Fix**: We develop a patch and tests.
4. **Review**: We review internally and optionally with external auditors.
5. **Release**: We release patched versions and security advisories.
6. **Disclosure**: We publish details after users have time to upgrade.

## Severity Guidelines

We use a simplified severity scale:

- **Critical**: private key compromise, signer manipulation, loss of funds, oracle corruption at scale
- **High**: replay protection bypass, consensus manipulation, major auth bypass
- **Medium**: limited privilege escalation, DoS with practical impact, data leakage
- **Low**: minor bugs with limited impact

## Security Practices

### Key Management

- Never commit private keys, seed phrases, or signing material.
- Prefer managed KMS (cloud KMS/HSM) for production signers.
- Rotate signer keys on a regular schedule and after any incident.
- Use separate keys for dev/staging/prod.

### Dependency Management

- Rust toolchain is pinned via `rust-toolchain.toml`.
- Rust dependencies are locked via `Cargo.lock` per crate/package.
- Node dependencies are locked via `package-lock.json` or equivalent.
- Use Dependabot or similar automation to keep dependencies current.

### CI and Supply Chain

- Use least-privilege GitHub Actions permissions.
- Pin actions to a commit SHA where practical.
- Use code scanning and secret scanning.
- Require reviews and status checks before merges.

### On-Chain Programs

- Prefer deterministic state transitions and explicit invariants.
- Use commit-reveal where front-running or manipulation is a concern.
- Ensure replay protection for all signed payloads.
- Keep signer-set rotation auditable and time-locked where feasible.
- Add fuzzing and property-based tests for instruction inputs.

### Services

- Enforce authentication and rate limiting for public endpoints.
- Avoid logging sensitive data.
- Use structured logging with request IDs.
- Apply strict input validation and safe defaults.

### Infrastructure

- Encrypt data at rest and in transit.
- Use network policies and least privilege IAM.
- Store secrets in a secret manager and inject at runtime.
- Enable monitoring and alerting for latency, error rates, and anomalous behavior.

## Vulnerability Rewards

We may offer discretionary rewards for responsibly disclosed vulnerabilities depending on impact and quality of report.
If you want to be considered for a reward, include a payout address and preferred contact details in your report.

## Security Updates and Advisories

When we release a security fix, we will:
- update `CHANGELOG.md`
- publish a GitHub Security Advisory (when applicable)
- tag a release

## Questions

If you have questions, contact:
- security@m0club.com

Links:
- Website: https://m0club.com/
- X: https://x.com/M0Clubonx
