#!/usr/bin/env python3
"""Release input gates and explicit, approval-protected draft operations."""
import argparse
import json
import hashlib
import os
from pathlib import Path
import re
import subprocess
import sys

from release_products import REPOSITORY, expected_assets, material, require, verify_candidate


def api(path, binary=False):
    command = ['gh', 'api', '--hostname', 'github.com', f'repos/{REPOSITORY}/{path}']
    if binary:
        command += ['-H', 'Accept: application/octet-stream']
    result = subprocess.run(command, check=True, stdout=subprocess.PIPE)
    return result.stdout if binary else json.loads(result.stdout)


def number(name):
    value = os.environ[name]
    require(bool(re.fullmatch('[1-9][0-9]*', value)), f'invalid {name}')
    return value


def sha(name):
    value = os.environ[name]
    require(bool(re.fullmatch('[0-9a-f]{64}', value)), f'invalid {name}')
    return value


def context():
    require(os.environ.get('GITHUB_REPOSITORY') == REPOSITORY, 'wrong repository')
    require(os.environ.get('GITHUB_REF') == 'refs/heads/main', 'dispatch must use main')


def successful_run(run_id, filename, commit=None):
    run = api(f'actions/runs/{run_id}')
    workflow = api(f'actions/workflows/{filename}')
    require(run['workflow_id'] == workflow['id'] and run['path'] == '.github/workflows/' + filename,
            'unexpected workflow identity')
    require(run['conclusion'] == 'success' and run['status'] == 'completed', 'run is not successfully completed')
    require(run['head_repository']['full_name'] == REPOSITORY, 'foreign source repository')
    require(re.fullmatch('[0-9a-f]{40}', run['head_sha']) is not None, 'invalid source identity')
    if commit is not None:
        require(run['head_sha'] == commit, 'CI or candidate belongs to a different source commit')
    return run


def candidate_inputs():
    context()
    p = material.policy()
    material.source_pin(p)
    commit = os.environ['GITHUB_SHA']
    require(re.fullmatch('[0-9a-f]{40}', commit) is not None, 'invalid workflow source commit')
    successful_run(number('CI_RUN_ID'), 'platform-ci.yml', commit)
    require(sha('PACK_SHA256') == p['archive_sha256'], 'archive digest is not approved in source')
    asset_id = number('PACK_ASSET_ID')
    asset = api(f'releases/assets/{asset_id}')
    require(asset['name'] == p['archive_name'] and 0 < asset['size'] <= material.LIMIT and asset['state'] == 'uploaded', 'unexpected proof asset')
    directory = Path('.release-ci/input')
    directory.mkdir(parents=True, exist_ok=False)
    archive = directory / p['archive_name']
    data = api(f'releases/assets/{asset_id}', binary=True)
    require(len(data) == asset['size'], 'proof asset length changed')
    with archive.open('xb') as stream:
        stream.write(data)
    require(material.digest(archive) == p['archive_sha256'], 'downloaded proof archive digest mismatch')
    # Validate the reviewed member set and hashes before sharing input bytes.
    material.extract(argparse.Namespace(archive=archive, sha256=p['archive_sha256'], output=Path('.release-ci/preflight')))
    with open(os.environ['GITHUB_OUTPUT'], 'a', encoding='utf-8') as stream:
        stream.write('source_sha=' + commit + '\n')


def release_operation():
    operation = os.environ.get('RELEASE_OPERATION', '')
    require(operation in ('create-draft', 'publish-draft'), 'explicit draft operation is required')
    verb = 'CREATE DRAFT' if operation == 'create-draft' else 'PUBLISH DRAFT'
    require(os.environ.get('CONFIRMATION') == verb + ' RETRO BOOMER v' + material.policy()['version'],
            'explicit operation-specific confirmation is missing')
    if operation == 'publish-draft':
        number('DRAFT_RELEASE_ID')
    else:
        require(not os.environ.get('DRAFT_RELEASE_ID'), 'draft creation must not select an existing release')
    return operation


def publication_inputs():
    context()
    release_operation()
    sha('CANDIDATE_SHA256')
    run = successful_run(number('CANDIDATE_RUN_ID'), 'release.yml')
    require(run['event'] == 'workflow_dispatch' and run['head_branch'] == 'main', 'candidate must be manually built from main')
    environment = api('environments/retro-boomer-release')
    require(any(rule.get('type') == 'required_reviewers' and rule.get('reviewers')
                for rule in environment.get('protection_rules', [])),
            'retro-boomer-release must have required reviewers configured by the owner')


def verify_publication(directory):
    context()
    operation = release_operation()
    require(material.digest(material.regular_file(directory / 'candidate.json')) == sha('CANDIDATE_SHA256'), 'candidate manifest differs from reviewed digest')
    data = verify_candidate(directory)
    require(data['candidate_run_id'] == number('CANDIDATE_RUN_ID'), 'candidate run identity mismatch')
    run = successful_run(data['candidate_run_id'], 'release.yml', data['source_commit'])
    require(run['event'] == 'workflow_dispatch' and run['head_branch'] == 'main', 'unexpected candidate origin')
    # A tag must already have been separately created/approved; this gate never creates it.
    tag = 'v' + data['version']
    ref = api('git/ref/tags/' + tag)['object']
    require(ref['type'] == 'tag', 'publication requires an existing annotated source tag')
    tagged = api('git/tags/' + ref['sha'])['object']
    require(tagged['type'] == 'commit' and tagged['sha'] == data['source_commit'], 'tag and approved candidate differ')
    notes = (directory / 'RELEASE_NOTES.md').read_text(encoding='utf-8')
    if operation == 'publish-draft':
        require('Release qualification: passed' in notes and 'preparation only' not in notes.lower(),
                'final reviewed release notes and qualification statement are required')
    return data


def verify_remote_draft(directory, data, release_id):
    release = api('releases/' + release_id)
    require(release['id'] == int(release_id) and release.get('draft') is True,
            'selected release is not the specified unpublished draft')
    require(release.get('tag_name') == 'v' + data['version']
            and release.get('target_commitish') == data['source_commit'], 'draft source or tag mismatch')
    require(release.get('name') == 'Retro Boomer v' + data['version'], 'unexpected draft title')
    require(release.get('body') == (directory / 'RELEASE_NOTES.md').read_text(encoding='utf-8'),
            'draft notes differ from the reviewed candidate')
    expected = dict(data['files'])
    expected['candidate.json'] = sha('CANDIDATE_SHA256')
    assets = release['assets']
    require(len(assets) == len(expected) and {a['name'] for a in assets} == set(expected),
            'draft contains missing, duplicate or unrelated assets')
    for asset in assets:
        name = asset['name']
        require(asset.get('state') == 'uploaded' and asset['size'] == (directory / name).stat().st_size,
                'draft asset size or state mismatch: ' + name)
        # Download only: stream large products without retaining them in RAM or rewriting files.
        process = subprocess.Popen(['gh', 'api', '--hostname', 'github.com',
                                    f"repos/{REPOSITORY}/releases/assets/{asset['id']}",
                                    '-H', 'Accept: application/octet-stream'], stdout=subprocess.PIPE)
        digest = hashlib.sha256()
        count = 0
        try:
            with process.stdout as stream:
                for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                    count += len(chunk)
                    require(count <= asset['size'], 'oversized draft download: ' + name)
                    digest.update(chunk)
            require(process.wait(timeout=120) == 0, 'draft asset download failed: ' + name)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
        require(count == asset['size'] and digest.hexdigest() == expected[name],
                'draft asset SHA-256 mismatch: ' + name)
    return release


def release_for_tag(tag):
    # Authenticated listing includes drafts; do not depend on CLI JSON field aliases.
    for page in range(1, 1001):
        releases = api(f'releases?per_page=100&page={page}')
        matches = [release for release in releases if release['tag_name'] == tag]
        require(len(matches) <= 1, 'ambiguous release tag')
        if matches:
            return matches[0]
        if len(releases) < 100:
            return None
    raise ValueError('release listing exceeded the bounded lookup limit')


def draft_action(directory):
    # Called only by the protected, contents:write job; verification never rebuilds.
    data = verify_publication(directory)
    operation = release_operation()
    tag = 'v' + data['version']
    if operation == 'create-draft':
        # gh refuses an existing release; never use --clobber or replace existing assets.
        require(release_for_tag(tag) is None, 'release tag already exists; refusing to alter it')
        paths = [str(directory / name) for name in sorted(set(data['files']) | {'candidate.json'})]
        subprocess.run(['gh', 'release', 'create', tag, *paths, '--repo', REPOSITORY,
                        '--draft', '--verify-tag', '--target', data['source_commit'],
                        '--title', 'Retro Boomer v' + data['version'],
                        '--notes-file', str(directory / 'RELEASE_NOTES.md')], check=True)
        # The tag was separately checked; locate this draft without creating another release.
        created = release_for_tag(tag)
        require(created is not None, 'created draft could not be located')
        release_id = str(created['id'])
        require(bool(re.fullmatch('[1-9][0-9]*', release_id)), 'invalid created draft ID')
        verify_remote_draft(directory, data, release_id)
        print('Created and verified unpublished draft ID:', release_id)
        summary = os.environ.get('GITHUB_STEP_SUMMARY')
        if summary:
            with open(summary, 'a', encoding='utf-8') as stream:
                stream.write('Unpublished final product draft ID: ' + release_id + '\n')
    else:
        release_id = number('DRAFT_RELEASE_ID')
        verify_remote_draft(directory, data, release_id)
        # Update exactly this verified draft ID. Never create a release on this path.
        result = subprocess.run(['gh', 'api', '--hostname', 'github.com', '--method', 'PATCH',
                                 f'repos/{REPOSITORY}/releases/{release_id}', '-F', 'draft=false'],
                                check=True, stdout=subprocess.PIPE)
        published = json.loads(result.stdout)
        require(published['id'] == int(release_id) and published.get('draft') is False,
                'publication response did not confirm the selected release')
        print('Published the verified existing draft ID:', release_id)



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['candidate-inputs','publication-inputs','verify-publication','draft-action','asset-names'])
    parser.add_argument('--directory', type=Path)
    args = parser.parse_args()
    try:
        if args.command == 'candidate-inputs': candidate_inputs()
        elif args.command == 'publication-inputs': publication_inputs()
        elif args.command == 'verify-publication':
            require(args.directory is not None, 'candidate directory required')
            verify_publication(args.directory)
        elif args.command == 'draft-action':
            require(args.directory is not None, 'candidate directory required')
            draft_action(args.directory)
        else:
            for name in sorted(expected_assets(material.policy()) | {'SHA256SUMS'}): print(name)
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'workflow gate: {error}\n')


if __name__ == '__main__':
    main()
