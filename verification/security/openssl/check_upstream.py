"""Check the latest stable patch release in KMIPKit's selected OpenSSL series."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

PIN_FILE = Path(__file__).with_name("native.json")
ISSUE_MARKER = "<!-- kmipkit-native-openssl-update -->"
ISSUE_TITLE_PREFIX = "OpenSSL native update available: "
USER_AGENT = "KMIPKit OpenSSL release tracker"


def load_pin() -> dict[str, str]:
    pin = json.loads(PIN_FILE.read_text(encoding="utf-8"))
    series = pin.get("series", "")
    if not re.fullmatch(r"\d+\.\d+", series):
        raise ValueError("native.json has an invalid OpenSSL release series")
    if not re.fullmatch(rf"{re.escape(series)}\.\d+", pin.get("version", "")):
        raise ValueError("native.json version does not match its OpenSSL release series")
    return pin


def get_json(url: str, token: str | None = None) -> object:
    headers = {"Accept": "application/vnd.github+json", "User-Agent": USER_AGENT}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def latest_patch(pin: dict[str, str]) -> tuple[str, str]:
    latest = tuple(map(int, pin["version"].split(".")))
    latest_tag = f"openssl-{pin['version']}"
    major, minor = map(int, pin["series"].split("."))
    api_url = pin["release_api_url"]
    for page in range(1, 6):
        releases = get_json(f"{api_url}?per_page=100&page={page}")
        if not isinstance(releases, list):
            raise ValueError("OpenSSL release API returned an unexpected response")
        for release in releases:
            if not isinstance(release, dict) or release.get("draft") or release.get("prerelease"):
                continue
            tag = release.get("tag_name", "")
            match = re.fullmatch(rf"openssl-{re.escape(pin['series'])}\.(\d+)", tag)
            if match:
                candidate = (major, minor, int(match.group(1)))
                if candidate > latest:
                    latest = candidate
                    latest_tag = tag
        if len(releases) < 100:
            break
    version = latest_tag.removeprefix("openssl-")
    return version, f"https://github.com/openssl/openssl/releases/tag/openssl-{version}"


def issue_body(pin: dict[str, str], latest: str, release_url: str) -> str:
    return (
        f"{ISSUE_MARKER}\n\n"
        "A newer stable patch release exists in KMIPKit's selected OpenSSL LTS series.\n\n"
        f"- Pinned version: `{pin['version']}`\n"
        f"- Available version: [`{latest}`]({release_url})\n"
        f"- Current archive SHA-256: `{pin['archive_sha256']}`\n"
        f"- OpenSSL source index: {pin['downloads_url']}\n\n"
        "Review the upstream release notes and official archive checksum. If the release is "
        "appropriate, update `verification/security/openssl/native.json`, rebuild the pinned "
        "source in CI, and update the OpenSSL build documentation in the same pull request. "
        "This tracker does not change the pinned version or checksum automatically.\n"
    )


def request_json(method: str, url: str, token: str, payload: dict[str, object]) -> dict[str, object]:
    data = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
            "User-Agent": USER_AGENT,
        },
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)
    if not isinstance(result, dict):
        raise ValueError("GitHub issue API returned an unexpected response")
    return result


def open_tracker_issue(token: str, repo: str, pin: dict[str, str], latest: str, release_url: str) -> None:
    base = f"https://api.github.com/repos/{repo}/issues"
    tracked: list[dict[str, object]] = []
    for page in range(1, 6):
        response = get_json(f"{base}?state=open&per_page=100&page={page}", token)
        if not isinstance(response, list):
            raise ValueError("GitHub issues API returned an unexpected response")
        tracked.extend(
            issue for issue in response
            if isinstance(issue, dict)
            and "pull_request" not in issue
            and ISSUE_MARKER in str(issue.get("body", ""))
        )
        if len(response) < 100:
            break

    title = f"{ISSUE_TITLE_PREFIX}{latest}"
    body = issue_body(pin, latest, release_url)
    if latest != pin["version"]:
        if tracked:
            issue = tracked[0]
            if issue.get("title") != title or issue.get("body") != body:
                request_json("PATCH", f"{base}/{issue['number']}", token, {"title": title, "body": body})
            print(f"Updated OpenSSL tracking issue: {issue.get('html_url')}")
        else:
            issue = request_json("POST", base, token, {"title": title, "body": body})
            print(f"Opened OpenSSL tracking issue: {issue.get('html_url')}")
    else:
        for issue in tracked:
            request_json("PATCH", f"{base}/{issue['number']}", token, {"state": "closed"})
            print(f"Closed resolved OpenSSL tracking issue: {issue.get('html_url')}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--report-issue",
        action="store_true",
        help="create/update one GitHub issue for an available patch, or close it when resolved",
    )
    args = parser.parse_args()
    pin = load_pin()
    latest, release_url = latest_patch(pin)
    newer = tuple(map(int, latest.split("."))) > tuple(map(int, pin["version"].split(".")))
    status = "update available" if newer else "up to date"
    message = f"OpenSSL {pin['series']} LTS: pinned {pin['version']}; latest stable patch {latest} ({status})."
    print(message)

    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with Path(summary).open("a", encoding="utf-8") as output:
            output.write(f"## Native OpenSSL update check\n\n{message}\n\n")
            output.write(f"Latest release: [{latest}]({release_url})\n\n")
            output.write(f"Pinned release: [{pin['version']}]({pin['release_url']})\n")

    if os.environ.get("GITHUB_OUTPUT"):
        with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as output:
            output.write(f"update_available={str(newer).lower()}\n")
            output.write(f"latest_version={latest}\n")

    if args.report_issue:
        token = os.environ.get("GITHUB_TOKEN")
        repo = os.environ.get("GITHUB_REPOSITORY")
        if not token or not repo:
            parser.error("--report-issue requires GITHUB_TOKEN and GITHUB_REPOSITORY")
        open_tracker_issue(token, repo, pin, latest, release_url)

    return 1 if newer and not args.report_issue else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except urllib.error.URLError as error:
        print(f"OpenSSL release check failed: {error}", file=sys.stderr)
        sys.exit(2)
