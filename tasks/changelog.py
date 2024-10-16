# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "gitpython>=3.1.43",
#   "pygithub>=2.4",
# ]
# ///
"""Generate the changelog on release."""

from __future__ import annotations

import os
import re
from argparse import ArgumentParser, Namespace
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING, Iterator

from git import Repo
from github import Github, Repository
from github.Auth import Token
from tomllib import load

if TYPE_CHECKING:
    from github.Repository import Repository as GitHubRepository

ROOT = Path(__file__).parents[1]


class Options(Namespace):
    project: str
    pr: int | None
    base: str


def run() -> None:
    options = parse_cli()
    print(options)
    project = ROOT / options.project
    changelog_file = project / "CHANGELOG.md"

    version = get_version(project)
    changelog = changelog_file.read_text(encoding="utf-8")
    anchor = f'<a id="{version}"></a>'

    logs = []
    git_repo = Repo(ROOT)
    github = Github(auth=Token(os.environ["GITHUB_TOKEN"]))
    at = "tox-dev/toml-fmt"
    gh_repo = github.get_repo(at)
    for title, pr, by in entries(gh_repo, git_repo, options.pr, options.base):
        suffix = f" in [#${pr}](https://github.com/{at}/pull/{pr})]" if pr else ""
        logs.append(f"{title} by [@{by}](https://github.com/{by}){suffix}")

    if logs:
        new_lines = [
            anchor,
            f"## {version} - {datetime.now(tz=UTC).date().isoformat()}",
            "",
            *[f" - {i}" for i in logs],
            "",
            "",
        ]
        new = "\n".join(new_lines)
        print(new)
        changelog_file.write_text(new + changelog)
    else:
        new = ""

    if output := os.environ.get("GITHUB_TOKEN"):
        with Path(output).open("at+", encoding="utf-8") as file_handler:
            file_handler.write(f"version={version}\n")
            file_handler.write(f"changelog<<EOF\n{new}\nEOF\n")


def parse_cli() -> Options:
    parser = ArgumentParser()
    parser.add_argument("project", choices=["pyproject-fmt"])
    parser.add_argument("pr", type=lambda s: int(s) if s else None)
    parser.add_argument("base", type=str)
    options = Options()
    parser.parse_args(namespace=options)
    return options


def get_version(base: Path) -> str:
    with (base / "Cargo.toml").open("rb") as cargo_toml_file_handler:
        return load(cargo_toml_file_handler)["package"]["version"]


def entries(
    gh_repo: GitHubRepository, git_repo: Repository, pr: int | None, base: str | None
) -> Iterator[tuple[str, str, str]]:
    if pr:
        pull = gh_repo.get_pull(pr)
        yield pull.title, str(pr), pull.user.login
    tags = {tag.commit.hexsha for tag in git_repo.tags}
    pr_re = re.compile(r"(?P<title>.*)[(]#(?P<pr>\d+)[)]")
    found_base = not base
    for change in git_repo.iter_commits():
        if change.hexsha in tags:
            break
        found_base = found_base or change.hexsha == base
        if not found_base or change.author.name in {"pre-commit-ci[bot]", "dependabot[bot]"}:
            continue
        title = change.message.split("\n")[0].strip()
        by = gh_repo.get_commit(change.hexsha).author.login
        if match := pr_re.match(title):
            group = match.groupdict()
            yield group["title"].strip(), group["pr"], by
        else:
            yield title, "", by


if __name__ == "__main__":
    run()
