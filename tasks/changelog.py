# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "gitpython>=3.1.46",
#   "prek>=0.3.1",
#   "pygithub>=2.8.1",
# ]
# ///
"""Generate the changelog on release."""

from __future__ import annotations

import os
import re
import subprocess  # noqa: S404
from argparse import ArgumentParser, Namespace
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING

import urllib3
from git import Repo
from github import Github, Repository
from github.Auth import Token
from tomllib import load

if TYPE_CHECKING:
    from collections.abc import Iterator

    from github.Repository import Repository as GitHubRepository

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

ROOT = Path(__file__).parents[1]


class Options(Namespace):
    project: str
    pr: int | None
    base: str
    regenerate: bool


def run() -> None:
    options = parse_cli()
    print(f">> {options}")
    project = ROOT / options.project
    changelog_file = project / "CHANGELOG.md"

    git_repo = Repo(ROOT)
    at = "tox-dev/toml-fmt"
    github = Github(auth=Token(os.environ["GITHUB_TOKEN"]), verify=False)
    gh_repo = github.get_repo(at)

    if options.regenerate:
        regenerate_changelog(changelog_file, git_repo, gh_repo, at, options.project)
        return

    version = get_version(project)
    changelog = changelog_file.read_text(encoding="utf-8")
    anchor = f'<a id="{version}"></a>'

    logs = []
    for title, pr, by in entries(gh_repo, git_repo, options.pr, options.base, options.project):
        suffix = f" in [#{pr}](https://github.com/{at}/pull/{pr})" if pr else ""
        logs.append(f"{title} by [@{by}](https://github.com/{by}){suffix}")

    if logs:
        logs = [f"- {i}" for i in logs]
        new_lines = [
            anchor,
            "",
            f"## {version} - {datetime.now(tz=UTC).date().isoformat()}",
            "",
            *logs,
            "",
        ]
        new = "\n".join(new_lines)
        print(new)
        logs_text = "\n".join(logs)
        changelog_file.write_text(new + changelog, encoding="utf-8")
        subprocess.run(["prek", "run", "--files", str(changelog_file)], check=False)  # noqa: S603, S607
    else:
        logs_text = ""

    if output := os.environ.get("GITHUB_OUTPUT"):
        print(f">> GitHub output set, populating: {output}")
        with Path(output).open("at+", encoding="utf-8") as file_handler:
            file_handler.write(f"version={version}\n")
            file_handler.write(f"changelog<<EOF\n{logs_text}\nEOF\n")


def parse_cli() -> Options:
    parser = ArgumentParser()
    parser.add_argument("project", choices=["pyproject-fmt", "tox-toml-fmt"])
    parser.add_argument("pr", type=lambda s: int(s) if s else None, nargs="?", default=None)
    parser.add_argument("base", type=str, nargs="?", default="")
    parser.add_argument("--regenerate", action="store_true", help="Regenerate entire changelog from all releases")
    options = Options()
    parser.parse_args(namespace=options)
    return options


def get_version(base: Path) -> str:
    with (base / "Cargo.toml").open("rb") as cargo_toml_file_handler:
        return load(cargo_toml_file_handler)["package"]["version"]


def regenerate_changelog(
    changelog_file: Path, git_repo: Repo, gh_repo: GitHubRepository, at: str, project: str
) -> None:
    project_tags = sorted(
        ((tag, tag.name.split("/")[1]) for tag in git_repo.tags if tag.name.startswith(f"{project}/")),
        key=lambda t: t[0].commit.committed_datetime,
    )
    if not project_tags:
        print(f"No tags found for {project}")
        return

    sections: list[str] = []
    for i, (tag, version) in enumerate(project_tags):
        prev_commit = project_tags[i - 1][0].commit.hexsha if i > 0 else None
        current_commit = tag.commit.hexsha
        release_date = tag.commit.committed_datetime.date().isoformat()

        logs = list(entries_between(gh_repo, git_repo, prev_commit, current_commit, at, project))
        if logs:
            log_lines = [f"- {entry}" for entry in logs]
            section = f'<a id="{version}"></a>\n\n## {version} - {release_date}\n\n' + "\n".join(log_lines)
            sections.append(section)
        else:
            sections.append(f'<a id="{version}"></a>\n\n## {version} - {release_date}\n\n- Initial release')

    content = "\n\n".join(reversed(sections))
    if project == "pyproject-fmt":
        content += (
            "\n\nFor versions before 2.4.0, see releases in the old repositories: "
            "[pyproject-fmt](https://github.com/tox-dev/pyproject-fmt/releases) (Python) and "
            "[pyproject-fmt-rust](https://github.com/tox-dev/pyproject-fmt-rust/releases) (Rust).\n"
        )
    else:
        content += "\n"
    changelog_file.write_text(content, encoding="utf-8")
    subprocess.run(["prek", "run", "--files", str(changelog_file)], check=False)  # noqa: S603, S607
    print(f"Regenerated changelog for {project} with {len(sections)} releases")


def entries_between(  # noqa: PLR0913, PLR0917
    gh_repo: GitHubRepository, git_repo: Repo, start: str | None, end: str, at: str, project: str
) -> Iterator[str]:
    pr_re = re.compile(r"(?P<title>.*)[(]#(?P<pr>\d+)[)]")
    release_re = re.compile(r"^Release \S+ \d+\.\d+\.\d+$")
    rev_range = f"{start}..{end}" if start else end
    for change in git_repo.iter_commits(rev_range):
        if change.author.name in {"pre-commit-ci[bot]", "dependabot[bot]"}:
            continue
        if not commit_affects_project(change, project):
            continue
        title = change.message.split("\n")[0].strip()
        if release_re.match(title):
            continue
        by = get_author_login(gh_repo, change)
        if match := pr_re.match(title):
            group = match.groupdict()
            pr = group["pr"]
            suffix = f" in [#{pr}](https://github.com/{at}/pull/{pr})"
            yield f"{group['title'].strip()} by [@{by}](https://github.com/{by}){suffix}"
        else:
            yield f"{title} by [@{by}](https://github.com/{by})"


def get_author_login(gh_repo: GitHubRepository, change: object) -> str:
    return gh_repo.get_commit(change.hexsha).author.login


def entries(
    gh_repo: GitHubRepository, git_repo: Repository, pr: int | None, base: str | None, project: str
) -> Iterator[tuple[str, str, str]]:
    if pr:
        pull = gh_repo.get_pull(pr)
        yield pull.title, str(pr), pull.user.login
    tags = {tag.commit.hexsha for tag in git_repo.tags if tag.name.startswith(f"{project}/")}
    pr_re = re.compile(r"(?P<title>.*)[(]#(?P<pr>\d+)[)]")
    release_re = re.compile(rf"^Release {re.escape(project)} \d+\.\d+\.\d+")
    found_base = not base
    for change in git_repo.iter_commits():
        if change.hexsha in tags:
            break
        title = change.message.split("\n")[0].strip()
        if release_re.match(title):
            break
        found_base = found_base or change.hexsha == base
        if not found_base or change.author.name in {"pre-commit-ci[bot]", "dependabot[bot]"}:
            continue
        if not commit_affects_project(change, project):
            continue
        by = get_author_login(gh_repo, change)
        if match := pr_re.match(title):
            group = match.groupdict()
            yield group["title"].strip(), group["pr"], by
        else:
            yield title, "", by


def commit_affects_project(commit: object, project: str) -> bool:
    changed_files = list(commit.stats.files.keys())
    return any(file_path.startswith(("common/", f"{project}/")) for file_path in changed_files)


if __name__ == "__main__":
    run()
