# Security Policy

## Supported versions

Only the latest release receives security fixes.

## Reporting a vulnerability

Do not open a public issue for a security problem.

- Discord: @flintsnow (direct message)
- Start the message with `Deadlock+ security report`.
- Include: affected version, steps to reproduce, and the impact you see.

Expect an acknowledgement within 7 days. Fixes are released as soon as they are ready, and you will be credited unless you ask not to be.

## Scope

In scope:

- Windows Firewall rule handling (only `deadlock_plus_*` rules should ever be changed or removed).
- The Task Scheduler start-with-Windows task.
- Handling of local files: mutes, replays, backups, logs, exports.
- The update mechanism: signature checks, the update manifest and the release workflow.
- Leaks of Steam IDs, IP addresses or other personal data.

Out of scope:

- Vulnerabilities in Deadlock, Steam, the Deadlock API or Windows themselves. Report those to their owners.
- Issues that need an attacker who already has administrator access to your PC.
