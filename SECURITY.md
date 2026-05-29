# Security Policy

## Supported Versions

RustGauge is currently a pre-1.0 personal project. Security fixes are handled on the latest `main` branch and current GitHub Releases.

## Reporting a Vulnerability

Please report security issues by opening a private security advisory on GitHub if available. If private advisories are not enabled, open an issue with minimal public detail and state that you have a security report to share.

Do not include sensitive logs, private machine details, or proof-of-concept code in a public issue.

## Privacy and Runtime Scope

RustGauge reads local system metrics and its own configuration. It does not send telemetry or make outbound network requests.

The `Start with Windows` option writes a per-user registry value under:

```text
HKCU\Software\Microsoft\Windows\CurrentVersion\Run
```

It does not require administrator privileges.
