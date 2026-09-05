# Security policy

## Reporting

Report vulnerabilities through the repository's **Security** tab using private
vulnerability reporting. Do not include sensitive details in a public issue.

If private reporting is unavailable, open an issue containing only a request
for a private channel. Include the affected commit, command, observed result,
expected result, and relevant profile or route in the eventual private report.
For capture defects, include the raw ClientHello bytes when safe to do so.

No response or remediation timeline is promised.

## Security model

`b-ids-harness` parses attacker-controlled TLS, HTTP/2, and HPACK bytes. It:

- binds to loopback by default;
- rejects hostnames and unspecified bind addresses;
- does not execute input;
- writes only to caller-selected paths; and
- is covered by fuzz targets and hostile-input tests.

Binding the harness to a non-loopback address creates a network service. This
project does not operate a hosted service.

The optional capture-oracle mode returns a caller's own capture. Nothing is
retained by default. With `--no-retain`, file-writing options such as `--ca-out`,
`--hello-out`, and `--write-golden` are rejected before a socket opens. Oracle
captures are not routed into the published corpus.

Header values are names-only by default. Credential-bearing fields such as
cookies and authorization headers are filtered, and the schema rejects known
credential shapes. Treat these checks as safeguards, not authorization to
collect private traffic.

## In scope

- parser panics, hangs, out-of-bounds access, or resource-exhaustion defects;
- published profiles that do not describe the browser build they name;
- credentials or private-system fingerprints reaching a published artefact;
- validation that reports success for a condition it is intended to reject;
- release, workflow, or data-branch behavior that violates immutability; and
- vulnerabilities in vendored code as integrated here.

Browser defects and misuse of correct published data are out of scope for this
repository. Report browser defects to the relevant vendor.

## Published data

The `source` and `data` branches are append-only evidence surfaces. A profile
that should not be public cannot be made private by deleting a later reference;
report suspected disclosure immediately.

Repository checks scan tracked first-party files for secrets. Workflows use
GitHub's job-scoped token and do not require a personal access token.
