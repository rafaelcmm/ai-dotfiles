---
name: devops-best-practices
description: DevOps practices for delivery speed, reliability, security, and operational excellence.
allowed-tools: [Read]
---

# DevOps Best Practices

Use DevOps principles to deliver software quickly and safely, with strong feedback loops and resilient operations.

## Purpose

- Shorten lead time from commit to production
- Improve deployment reliability and recovery speed
- Standardize secure, observable delivery workflows
- Align development and operations ownership

## When to Reference This Skill

Reference when:

- Designing CI/CD pipelines
- Improving deployment reliability
- Defining infrastructure and environment strategy
- Implementing monitoring, alerting, or incident response

## Core Pillars

1. **Automation first**: build, test, scan, release, and rollback
2. **Shift-left quality/security**: catch issues early in pipeline
3. **Observability by default**: logs, metrics, traces, SLOs
4. **Progressive delivery**: feature flags, canary, blue-green
5. **Infrastructure as code**: versioned, reviewable environments

## CI/CD Baseline

```text
Commit -> Lint/Test -> Security Scan -> Build Artifact -> Deploy Staging
-> Smoke Tests -> Approval/Policy Gate -> Production Deploy -> Post-Deploy Checks
```

## Operational Standards

- Define SLI/SLO per critical service
- Use runbooks for common incidents
- Track MTTR and deployment failure rate
- Automate backups and restore drills
- Enforce least privilege and secret rotation

## Security Practices

- Pin and scan dependencies
- Scan containers and IaC templates
- Sign artifacts and verify provenance
- Keep secrets out of source control
- Patch base images and runtimes regularly

## Quick Checklist

```text
- [ ] Pipeline is fully automated and reproducible
- [ ] Rollback strategy is tested
- [ ] Telemetry exists for user-critical paths
- [ ] Security scans block high-severity issues
- [ ] On-call runbooks are current
```
