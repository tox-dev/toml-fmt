# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "gitpython>=3.1.46",
#   "prek>=0.3.1",
#   "pygithub>=2.8.1",
# ]
# ///
"""Build release notes from commits that affect a distributable package."""

from __future__ import annotations

import os
import re
from argparse import ArgumentParser, Namespace
from pathlib import Path
from typing import TYPE_CHECKING, Final

import urllib3
from git import Repo
from github import Github
from github.Auth import Token
from local_inputs import LOCAL_INPUTS, affects
from tomllib import load

if TYPE_CHECKING:
    from collections.abc import Iterator, Sequence

    from github.Repository import Repository as GitHubRepository

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

_ROOT: Final[Path] = Path(__file__).parents[1]


class _Options(Namespace):
    project: str
    pr: int | None
    base: str


def _main() -> None:
    options = _parse_cli()
    print(f">> {options}")
    project = _ROOT / options.project

    repository_name = "tox-dev/toml-fmt"
    titles = [
        (title, pr)
        for title, pr, _ in _entries(
            Github(auth=Token(os.environ["GITHUB_TOKEN"]), verify=False).get_repo(repository_name),
            Repo(_ROOT),
            options.pr,
            options.base,
            options.project,
        )
    ]
    notes = _release_notes(titles, repository_name)
    print(notes)

    if output := os.environ.get("GITHUB_OUTPUT"):
        print(f">> write GitHub output: {output}")
        with Path(output).open("at+", encoding="utf-8") as file_handler:
            file_handler.write(f"version={_get_version(project)}\n")
            file_handler.write(f"changelog<<EOF\n{notes}\nEOF\n")


def _parse_cli() -> _Options:
    parser = ArgumentParser()
    parser.add_argument("project", choices=sorted(LOCAL_INPUTS))
    parser.add_argument("pr", type=lambda s: int(s) if s else None, nargs="?", default=None)
    parser.add_argument("base", type=str, nargs="?", default="")
    options = _Options()
    parser.parse_args(namespace=options)
    return options


def _get_version(base: Path) -> str:
    if (cargo := base / "Cargo.toml").exists():
        with cargo.open("rb") as file:
            return load(file)["package"]["version"]
    with (base / "pyproject.toml").open("rb") as file:
        return load(file)["project"]["version"]


def _entries(
    gh_repo: GitHubRepository, git_repo: Repo, pr: int | None, base: str | None, project: str
) -> Iterator[tuple[str, str, str]]:
    if pr:
        pull = gh_repo.get_pull(pr)
        yield pull.title, str(pr), pull.user.login
    tags = {tag.commit.hexsha for tag in git_repo.tags if tag.name.startswith(f"{project}/")}
    pr_pattern = re.compile(r"(?P<title>.*)[(]#(?P<pr>\d+)[)]")
    release_pattern = re.compile(rf"^Release {re.escape(project)} \d+\.\d+\.\d+")
    found_base = not base
    for change in git_repo.iter_commits():
        if change.hexsha in tags:
            break
        title = change.message.split("\n")[0].strip()
        if release_pattern.match(title):
            break
        found_base = found_base or change.hexsha == base
        if not found_base or change.author.name in {"pre-commit-ci[bot]", "dependabot[bot]"}:
            continue
        if not affects(project, change.stats.files):
            continue
        login = gh_repo.get_commit(change.hexsha).author.login
        if match := pr_pattern.match(title):
            group = match.groupdict()
            yield group["title"].strip(), group["pr"], login
        else:
            yield title, "", login


# Commit types absent from this map describe internal changes that release notes omit.
_HEADINGS: Final[dict[str, str]] = {
    "feat": "Added",
    "fix": "Fixed",
    "perf": "Performance",
    "docs": "Documentation",
}

# The release template places additions before repairs.
_HEADING_ORDER: Final[tuple[str, ...]] = (
    "Added",
    "Changed",
    "Fixed",
    "Removed",
    "Performance",
    "Documentation",
    "Packaging",
)


def _release_notes(titles: Sequence[tuple[str, str | None]], repository_name: str) -> str:
    grouped: dict[str, list[str]] = {}
    for title, pr in titles:
        if (heading := _heading_of(title)) is None:
            continue
        entry = f"- **{_description(title)}**"
        grouped.setdefault(heading, []).append(
            f"{entry} ([#{pr}](https://github.com/{repository_name}/pull/{pr}))" if pr else entry
        )
    return "\n\n".join(
        f"### {heading}\n\n" + "\n".join(grouped[heading]) for heading in _HEADING_ORDER if heading in grouped
    )


def _heading_of(title: str) -> str | None:
    if (match := re.match(r"^\W*\s*(\w+)(\([^)]*\))?:", title)) is None:
        return None
    return _HEADINGS.get(match.group(1).lower())


def _description(title: str) -> str:
    description = re.sub(r"^\W*\s*\w+(\([^)]*\))?:\s*", "", title).strip()
    return description[:1].upper() + description[1:] + ("" if description.endswith(".") else ".")


if __name__ == "__main__":
    _main()
