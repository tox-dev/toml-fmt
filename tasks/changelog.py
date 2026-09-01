# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "gitpython>=3.1.46",
#   "prek>=0.3.1",
#   "pygithub>=2.8.1",
# ]
# ///
"""The notes a release goes out with."""

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
    from collections.abc import Iterator

    from github.Repository import Repository as GitHubRepository

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

_ROOT: Final[Path] = Path(__file__).parents[1]


class Options(Namespace):
    project: str
    pr: int | None
    base: str


def run() -> None:
    options = parse_cli()
    print(f">> {options}")
    project = _ROOT / options.project

    git_repo = Repo(_ROOT)
    at = "tox-dev/toml-fmt"
    github = Github(auth=Token(os.environ["GITHUB_TOKEN"]), verify=False)
    gh_repo = github.get_repo(at)

    version = get_version(project)
    titles = [(title, pr) for title, pr, _ in entries(gh_repo, git_repo, options.pr, options.base, options.project)]
    notes = release_notes(titles, at)
    print(notes)

    if output := os.environ.get("GITHUB_OUTPUT"):
        print(f">> GitHub output set, populating: {output}")
        with Path(output).open("at+", encoding="utf-8") as file_handler:
            file_handler.write(f"version={version}\n")
            file_handler.write(f"changelog<<EOF\n{notes}\nEOF\n")


def parse_cli() -> Options:
    parser = ArgumentParser()
    parser.add_argument("project", choices=sorted(LOCAL_INPUTS))
    parser.add_argument("pr", type=lambda s: int(s) if s else None, nargs="?", default=None)
    parser.add_argument("base", type=str, nargs="?", default="")
    options = Options()
    parser.parse_args(namespace=options)
    return options


def get_version(base: Path) -> str:
    if (cargo := base / "Cargo.toml").exists():
        with cargo.open("rb") as file:
            return load(file)["package"]["version"]
    with (base / "pyproject.toml").open("rb") as file:
        return load(file)["project"]["version"]


def entries(
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


#: The heading a change goes under, read from the conventional-commit type its title opens with. A
#: type nothing here names is internal churn, and notes a user reads leave it out.
UNDER: Final[dict[str, str]] = {"feat": "Added", "fix": "Fixed", "perf": "Performance", "docs": "Documentation"}

#: The order the headings read in, so every release says what it added before what it repaired.
IN_ORDER: Final[tuple[str, ...]] = ("Added", "Changed", "Fixed", "Removed", "Performance", "Documentation", "Packaging")


def release_notes(titles: list[tuple[str, str | None]], at: str) -> str:
    """What the release says it changed, grouped under the heading each change belongs to."""
    grouped: dict[str, list[str]] = {}
    for title, pr in titles:
        if (under := heading_of(title)) is None:
            continue
        said = f"- **{described(title)}**"
        grouped.setdefault(under, []).append(f"{said} ([#{pr}](https://github.com/{at}/pull/{pr}))" if pr else said)
    return "\n\n".join(f"### {under}\n\n" + "\n".join(grouped[under]) for under in IN_ORDER if under in grouped)


def heading_of(title: str) -> str | None:
    """The heading the title belongs under, or `None` where it says nothing a user reads."""
    if (match := re.match(r"^\W*\s*(\w+)(\([^)]*\))?:", title)) is None:
        return None
    return UNDER.get(match.group(1).lower())


def described(title: str) -> str:
    """What the title says, without the emoji and the `type(scope):` the commit convention adds."""
    said = re.sub(r"^\W*\s*\w+(\([^)]*\))?:\s*", "", title).strip()
    return said[:1].upper() + said[1:] + ("" if said.endswith(".") else ".")


if __name__ == "__main__":
    run()
