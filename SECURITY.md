# Security Policy

## Supported Versions

We release security updates for the following versions of YEET:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please follow these steps:

### 1. **DO NOT** Open a Public Issue

Please **do not** open a public GitHub issue for security vulnerabilities, as this could put users at risk before a fix is available.

### 2. Report Privately

Send an email to: **security@yeet.sh** (or the maintainer's email if this doesn't exist)

Include the following information:

- **Description**: Clear description of the vulnerability
- **Impact**: What an attacker could achieve
- **Steps to Reproduce**: Detailed steps to reproduce the issue
- **Proof of Concept**: Code or commands demonstrating the vulnerability
- **Affected Versions**: Which versions are affected
- **Suggested Fix**: If you have ideas for remediation

### 3. Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Fix Development**: Depends on severity (see below)
- **Public Disclosure**: After fix is released and users have time to upgrade

## Severity Levels

We classify vulnerabilities using the following severity levels:

### Critical (CVSS 9.0-10.0)
- Remote code execution
- Authentication bypass
- Data breach potential

**Response Time**: Fix within 7 days

### High (CVSS 7.0-8.9)
- Privilege escalation
- Path traversal with significant impact
- XSS leading to account compromise

**Response Time**: Fix within 14 days

### Medium (CVSS 4.0-6.9)
- Information disclosure
- Denial of service
- XSS with limited impact

**Response Time**: Fix within 30 days

### Low (CVSS 0.1-3.9)
- Minor information leaks
- Non-exploitable edge cases

**Response Time**: Fix in next regular release

## Known Security Considerations

### Cloudflare Tunnel URLs

YEET uses Cloudflare Quick Tunnels which generate temporary, random URLs. These URLs are:

- **Not secret**: Anyone with the URL can access your shared files
- **Temporary**: URLs change if daemon restarts
- **No authentication**: By design (zero-config)

**Recommendation**: Only share files you're comfortable making temporarily public. Do not share sensitive data without additional encryption.

### Admin Dashboard

The `/admin` dashboard is publicly accessible if you share your tunnel URL. It shows:

- Server uptime and statistics
- Request logs (IPs, user agents, paths)

**Recommendation**: Treat the admin dashboard as public information. It's designed for local development and trusted networks.

### State File

The tunnel state file (`~/.yeet/tunnel.state`) contains your tunnel URL. As of version 0.1.3+, this file has restricted permissions (0o600).

**Recommendation**: Ensure your home directory has appropriate permissions.

## Security Best Practices

When using YEET:

1. **Only share files you trust**: Anyone with the tunnel URL can access your files
2. **Use temporary shares**: Don't leave daemons running indefinitely
3. **Monitor the admin dashboard**: Check who's accessing your files
4. **Kill tunnels when done**: Use `yeet --kill` to stop sharing
5. **Don't share sensitive data**: Use encryption if sharing confidential files
6. **Keep YEET updated**: Install security updates promptly

## Security Features

### Current (as of Unreleased)

- ✅ Path traversal prevention
- ✅ HTML escaping to prevent XSS
- ✅ Restricted file permissions for state files (0o600)
- ✅ Input validation and sanitization
- ✅ Streaming large files (prevents memory exhaustion)
- ✅ Localhost-only HTTP server (127.0.0.1)
- ✅ HTTPS via Cloudflare tunnels

### Future Enhancements

- 🔄 Optional authentication for admin dashboard
- 🔄 Optional password protection for shared files
- 🔄 Rate limiting
- 🔄 File access logging
- 🔄 Configurable file size limits
- 🔄 Whitelist/blacklist for file types

## Past Vulnerabilities

### [CVE-TBD] Path Traversal (Fixed in Unreleased)

**Severity**: Critical (CVSS 9.1)

**Description**: Directory traversal vulnerability allowed attackers to access files outside the shared directory using `/../../../` sequences.

**Affected Versions**: ≤ 0.1.2

**Fixed In**: Unreleased

**Mitigation**: Upgrade to latest version

### [CVE-TBD] XSS in Directory Listings (Fixed in Unreleased)

**Severity**: High (CVSS 7.3)

**Description**: Filenames in directory listings were not HTML-escaped, allowing XSS attacks through specially crafted filenames.

**Affected Versions**: ≤ 0.1.2

**Fixed In**: Unreleased

**Mitigation**: Upgrade to latest version

## Security Audit History

| Date | Auditor | Scope | Findings |
|------|---------|-------|----------|
| 2025-11-20 | Internal | Full codebase | 2 Critical, 1 High, 3 Medium (all fixed) |

## Responsible Disclosure

We follow coordinated vulnerability disclosure:

1. Reporter notifies us privately
2. We acknowledge and investigate
3. We develop and test a fix
4. We release the fix
5. We publish a security advisory
6. After 90 days (or when 95% of users have upgraded), we publish full details

## Bug Bounty Program

We currently do not offer a bug bounty program. However, we deeply appreciate security researchers who report vulnerabilities responsibly and will acknowledge your contribution in our security advisories (with your permission).

## Security Contacts

- **Primary**: security@yeet.sh (if configured)
- **Maintainer**: [GitHub profile](https://github.com/akash-otonomy)
- **GPG Key**: (Add if available)

## Additional Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Cloudflare Tunnel Security](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/)

---

**Thank you for helping keep YEET secure!** 🔒
