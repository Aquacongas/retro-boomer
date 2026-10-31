#!/usr/bin/env python3
"""Audit native products, package Core and seal/recheck a non-publishing candidate."""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tarfile
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import release_material as material

PLATFORMS = {
    'linux-x86_64': ('x86_64-unknown-linux-gnu', '.tar.gz', '.deb'),
    'linux-aarch64': ('aarch64-unknown-linux-gnu', '.tar.gz', '.deb'),
    'windows-x86_64': ('x86_64-pc-windows-msvc', '.zip', '-setup.exe'),
    'macos-x86_64': ('x86_64-apple-darwin', '.tar.gz', '.dmg'),
    'macos-aarch64': ('aarch64-apple-darwin', '.tar.gz', '.dmg'),
}
def regular_file(path):
    path = Path(path)
    if path.is_symlink() or not path.is_file() or not 0 < path.stat().st_size <= 2 * 1024**3:
        raise ValueError(f'invalid or oversized product: {path}')
    return path


GENESIS = 'cede1246070229c7d312f341a27d801dbf5b7bf32f8c76552d21f9203b325a13'
REPOSITORY = 'Aquacongas/retro-boomer'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def names(platform, version):
    _, core, gui = PLATFORMS[platform]
    return [f'retro-boomer-core-v{version}-{platform}{core}', f'retro-boomer-gui-v{version}-{platform}{gui}']


def architecture(path, platform):
    with regular_file(path).open('rb') as stream:
        header = stream.read(64)
        if platform.startswith('linux-'):
            require(header[:6] == b'\x7fELF\x02\x01', 'expected little-endian ELF64')
            machine = int.from_bytes(header[18:20], 'little')
            require(machine == (62 if platform.endswith('x86_64') else 183), 'ELF architecture mismatch')
        elif platform.startswith('windows-'):
            require(header[:2] == b'MZ', 'expected PE image')
            stream.seek(int.from_bytes(header[60:64], 'little'))
            pe = stream.read(26)
            require(pe[:6] == b'PE\x00\x00\x64\x86' and pe[24:26] == b'\x0b\x02', 'expected AMD64 PE32+')
        else:
            require(header[:4] == b'\xcf\xfa\xed\xfe', 'expected thin little-endian Mach-O64')
            cpu = int.from_bytes(header[4:8], 'little')
            require(cpu == (0x01000007 if platform.endswith('x86_64') else 0x0100000c), 'Mach-O architecture mismatch')


def binaries(directory, platform):
    suffix = '.exe' if platform.startswith('windows-') else ''
    files = [directory / (n + suffix) for n in ('retro-boomerd', 'retro-boomer-cli', 'retro-boomer-miner', 'retro-boomer-gui')]
    for f in files:
        architecture(f, platform)
    return files


def matrix_record(directory):
    p = material.policy()
    candidates = []
    for f in directory.rglob('matrix-checks.json'):
        record = json.loads(f.read_text())
        if record.get('bank') == p['bank']:
            candidates.append(record)
    require(len(candidates) == 1, 'expected exactly one authenticated node matrix record in the fresh target directory')
    record = candidates[0]
    require(record.get('schema') == 1 and record.get('metadata_sha256') == p['files']['v21-runtime-metadata.bin'], 'matrix metadata identity mismatch')
    expected = {n: h for n, h in p['files'].items() if n.endswith('.zst')}
    matrices = record.get('matrices', [])
    require(len(matrices) == 3 and {m['file']: m['compressed_sha256'] for m in matrices} == expected, 'incomplete or mismatched matrix authentication record')
    return record


def validate_smoke_report(report):
    require(report.get('genesis') == GENESIS and report.get('isolated') is True,
            'candidate genesis or isolation smoke missing')
    require(report.get('native_runtime') == 'ready' and report.get('activation_height') == 1,
            'candidate block-one runtime smoke missing')
    require(report.get('mining') is False and report.get('issuance_units') == '10000000000000000',
            'candidate mining or issuance smoke mismatch')


def document_sources(platform):
    root = Path(__file__).resolve().parents[2]
    sources = {'LICENSE': root / 'LICENSE', 'NOTICE': root / 'NOTICE',
               'README.txt': root / 'scripts/release/README.txt'}
    if platform.startswith('linux-'):
        sources['retro-boomerd.service'] = root / 'scripts/release/gui/linux/retro-boomerd.service'
    return sources


def record(args):
    p = material.policy()
    files = binaries(args.bin_dir, args.platform)
    commit = os.environ.get('RETRO_BOOMER_SOURCE_COMMIT') or None
    require(commit is None or re.fullmatch('[0-9a-f]{40}', commit), 'invalid source commit')
    data = {'schema': 1, 'project': 'Retro Boomer', 'version': p['version'], 'platform': args.platform,
            'target': PLATFORMS[args.platform][0], 'source_commit': commit,
            'bank': p['bank'], 'proof_archive_sha256': p['archive_sha256'],
            'binaries': {f.name: material.digest(f) for f in files},
            'documents': {name: material.digest(path) for name, path in document_sources(args.platform).items()},
            'matrix_authentication': matrix_record(args.target_dir),
            'runtime_smoke': json.loads(args.smoke.read_text())}
    validate_smoke_report(data['runtime_smoke'])
    args.output.write_bytes(material.json_bytes(data))


def core(args):
    p = material.policy()
    files = binaries(args.bin_dir, args.platform)[:3]
    root = Path(__file__).resolve().parents[2]
    sources = {f.name: f for f in files}
    sources.update(document_sources(args.platform))
    sources['BUILD.json'] = args.record
    name = names(args.platform, p['version'])[0]
    stem = name.removesuffix('.tar.gz').removesuffix('.zip')
    destination = args.output / name
    args.output.mkdir(parents=True, exist_ok=True)
    if name.endswith('.zip'):
        with zipfile.ZipFile(destination, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
            for name, path in sorted(sources.items()):
                info = zipfile.ZipInfo(stem + '/' + name, (1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.create_system = 3
                info.external_attr = (0o100755 if name.endswith('.exe') else 0o100644) << 16
                archive.writestr(info, path.read_bytes())
    else:
        with destination.open('xb') as raw, gzip.GzipFile(fileobj=raw, filename='', mode='wb', mtime=0) as gz:
            with tarfile.open(fileobj=gz, mode='w', format=tarfile.USTAR_FORMAT) as archive:
                for name, path in sorted(sources.items()):
                    data = path.read_bytes()
                    info = tarfile.TarInfo(stem + '/' + name)
                    info.mode = 0o755 if name in ('retro-boomerd', 'retro-boomer-cli', 'retro-boomer-miner') else 0o644
                    info.size = len(data)
                    archive.addfile(info, io.BytesIO(data))
    verify_core(destination, args.platform, json.loads(args.record.read_text()))
    print(destination)


def verify_core(path, platform, record):
    p = material.policy()
    suffix = '.exe' if platform.startswith('windows-') else ''
    binary_names = {n + suffix for n in ('retro-boomerd', 'retro-boomer-cli', 'retro-boomer-miner')}
    expected = binary_names | set(document_sources(platform)) | {'BUILD.json'}
    stem = names(platform, p['version'])[0].removesuffix('.tar.gz').removesuffix('.zip')
    hashes = {name: record['binaries'][name] for name in binary_names}
    hashes.update(record['documents'])
    hashes['BUILD.json'] = hashlib.sha256(material.json_bytes(record)).hexdigest()
    def check_member(name, size, stream):
        require(name.startswith(stem + '/') and name[len(stem)+1:] in expected, 'unexpected Core member')
        short = name[len(stem)+1:]
        require(short not in seen, 'duplicate Core member')
        require(0 < size <= 2 * 1024**3, 'invalid Core member size')
        h = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024*1024), b''): h.update(chunk)
        require(h.hexdigest() == hashes[short], 'Core payload hash mismatch: ' + short)
        return short
    seen = []
    if platform.startswith('windows-'):
        with zipfile.ZipFile(path) as archive:
            for member in archive.infolist():
                require(not member.is_dir() and (member.external_attr >> 16) & 0o170000 != 0o120000, 'unsafe Core ZIP member')
                with archive.open(member) as stream: seen.append(check_member(member.filename,member.file_size,stream))
    else:
        with tarfile.open(path,'r:gz') as archive:
            for member in archive:
                require(member.isfile(), 'unsafe Core tar member')
                with archive.extractfile(member) as stream: seen.append(check_member(member.name,member.size,stream))
    require(len(seen) == len(expected) and set(seen) == expected, 'incomplete or duplicate Core payload')


def checksums(directory, files):
    return ''.join(f'{material.digest(regular_file(directory / n))}  {n}\n' for n in sorted(files))


def expected_assets(p):
    return {n for platform in PLATFORMS for n in names(platform, p['version'])} | {p['archive_name']}


def assemble(args):
    p = material.policy()
    require(re.fullmatch('[0-9a-f]{40}', args.source_commit), 'candidate requires a real source commit')
    require(args.run_id.isdecimal() and int(args.run_id) > 0, 'invalid candidate run ID')
    require(material.digest(regular_file(args.pack)) == p['archive_sha256'], 'candidate pack mismatch')
    require(not args.output.exists(), 'candidate output already exists')
    expected = {n for platform in PLATFORMS for n in (*names(platform, p['version']), f'BUILD-{platform}.json')}
    files = [f for f in args.inputs.rglob('*') if f.is_file()]
    require(len(files) == len(expected) and {f.name for f in files} == expected, 'expected exactly ten products and five build records')
    args.output.mkdir(parents=True)
    for f in files:
        regular_file(f)
        shutil.copyfile(f, args.output / f.name)
    shutil.copyfile(args.pack, args.output / p['archive_name'])
    shutil.copyfile(Path(__file__).resolve().parents[2] / '.github/release-notes/v1.0.0.md', args.output / 'RELEASE_NOTES.md')
    (args.output / 'SHA256SUMS').write_text(checksums(args.output, expected_assets(p)), encoding='ascii')
    data = {'schema': 1, 'project': 'Retro Boomer', 'repository': REPOSITORY, 'version': p['version'],
            'source_commit': args.source_commit, 'candidate_run_id': args.run_id,
            'bank': p['bank'], 'proof_archive_sha256': p['archive_sha256'],
            'files': {f.name: material.digest(f) for f in sorted(args.output.iterdir())}}
    (args.output / 'candidate.json').write_bytes(material.json_bytes(data))
    verify_candidate(args.output)
    print('Review candidate.json SHA-256:', material.digest(args.output / 'candidate.json'))


def verify_candidate(directory):
    p = material.policy()
    data = json.loads(regular_file(directory / 'candidate.json').read_text())
    require(data.get('schema') == 1 and data.get('repository') == REPOSITORY and data.get('version') == p['version'], 'unexpected candidate identity')
    require(re.fullmatch('[0-9a-f]{40}', data['source_commit']) and data['candidate_run_id'].isdecimal() and int(data['candidate_run_id']) > 0, 'invalid candidate source or run')
    require(data.get('bank') == p['bank'] and data.get('proof_archive_sha256') == p['archive_sha256'], 'candidate bank mismatch')
    records = {f'BUILD-{platform}.json' for platform in PLATFORMS}
    required = expected_assets(p) | records | {'SHA256SUMS', 'RELEASE_NOTES.md'}
    require(set(data['files']) == required and {f.name for f in directory.iterdir()} == required | {'candidate.json'}, 'candidate file set mismatch')
    for name, digest in data['files'].items():
        require(material.digest(regular_file(directory / name)) == digest, f'candidate digest mismatch: {name}')
    require((directory / 'SHA256SUMS').read_text() == checksums(directory, expected_assets(p)), 'candidate release checksums mismatch')
    require(material.digest(directory / p['archive_name']) == p['archive_sha256'], 'candidate proof archive mismatch')
    for platform in PLATFORMS:
        record = json.loads((directory / f'BUILD-{platform}.json').read_text())
        require(record.get('schema') == 1 and record.get('project') == 'Retro Boomer'
                and record.get('version') == p['version'], 'candidate build identity mismatch')
        require(record['platform'] == platform and record['target'] == PLATFORMS[platform][0], 'candidate architecture record mismatch')
        require(record['source_commit'] == data['source_commit'] and record['bank'] == p['bank'] and record['proof_archive_sha256'] == p['archive_sha256'], 'mixed source or bank')
        require(set(record['documents']) == set(document_sources(platform)) and all(re.fullmatch('[0-9a-f]{64}', h) for h in record['documents'].values()), 'wrong document record set')
        expected_binaries = {n + ('.exe' if platform.startswith('windows-') else '') for n in ('retro-boomerd','retro-boomer-cli','retro-boomer-miner','retro-boomer-gui')}
        require(set(record['binaries']) == expected_binaries and all(re.fullmatch('[0-9a-f]{64}', h) for h in record['binaries'].values()), 'wrong binary record set')
        matrix = record['matrix_authentication']
        require(matrix.get('schema') == 1 and matrix['bank'] == p['bank'] and matrix['metadata_sha256'] == p['files']['v21-runtime-metadata.bin'], 'matrix report bank mismatch')
        require(len(matrix['matrices']) == 3 and {m['file']: m['compressed_sha256'] for m in matrix['matrices']} == {n:h for n,h in p['files'].items() if n.endswith('.zst')}, 'incomplete matrix report')
        verify_core(directory / names(platform, p['version'])[0], platform, record)
        validate_smoke_report(record['runtime_smoke'])
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest='command', required=True)
    for command in ('check-binaries', 'record', 'core'):
        c = subs.add_parser(command)
        c.add_argument('--bin-dir', type=Path, required=True)
        c.add_argument('--platform', choices=PLATFORMS, required=True)
        if command == 'record':
            c.add_argument('--target-dir', type=Path, required=True); c.add_argument('--smoke', type=Path, required=True); c.add_argument('--output', type=Path, required=True)
        if command == 'core':
            c.add_argument('--record', type=Path, required=True); c.add_argument('--output', type=Path, required=True)
    c = subs.add_parser('source-value'); c.add_argument('--name', choices=['toolchain','version'], required=True)
    c = subs.add_parser('assemble')
    for n in ('inputs','pack','output'): c.add_argument('--'+n, type=Path, required=True)
    c.add_argument('--source-commit', required=True); c.add_argument('--run-id', required=True)
    c = subs.add_parser('verify-candidate'); c.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == 'source-value':
            root = Path(__file__).resolve().parents[2]
            if args.name == 'toolchain':
                print(re.search(r'channel = "([^"]+)"', (root / 'rust-toolchain.toml').read_text())[1])
            else:
                version = re.search(r'\[workspace.package\]\s+version = "([^"]+)"', (root / 'Cargo.toml').read_text())[1]
                require(version == material.policy()['version'], 'workspace and release policy version mismatch')
                print(version)
        elif args.command == 'check-binaries': binaries(args.bin_dir,args.platform)
        elif args.command == 'record': record(args)
        elif args.command == 'core': core(args)
        elif args.command == 'assemble': assemble(args)
        else: verify_candidate(args.directory); print('Candidate identity, hashes, platforms and bank verified; no compilation performed.')
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f'release products: {error}\n')


if __name__ == '__main__':
    main()
