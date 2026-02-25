# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

Please report (suspected) security vulnerabilities to the project maintainer. You will receive a response within 48 hours. If the issue is confirmed, we will release a patch as soon as possible depending on complexity.

### What to include in your report

* Type of issue (e.g. buffer overflow, SQL injection, cross-site scripting, etc.)
* Full paths of source file(s) related to the manifestation of the issue
* The location of the affected source code (tag/branch/commit or direct URL)
* Any special configuration required to reproduce the issue
* Step-by-step instructions to reproduce the issue
* Proof-of-concept or exploit code (if possible)
* Impact of the issue, including how an attacker might exploit the issue

### What to expect

* We will respond within 48 hours acknowledging receipt of your vulnerability report
* We will work with you to understand and address the issue
* We will keep you informed of our progress
* We will credit you in the security advisory (unless you prefer to remain anonymous)

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any potential similar problems
3. Prepare fixes for all releases still under maintenance
4. Release new security fix versions as soon as possible

Please do not disclose the vulnerability publicly until we have had a chance to address it.

## Comments on this Policy

If you have suggestions on how this process could be improved, please submit a pull request.
