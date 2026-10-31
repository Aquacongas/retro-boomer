#!/usr/bin/env python3
"""Deterministically package approved native RETRO BOOMER distribution material; never generate or prove."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import re
import tarfile

ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / 'scripts/release/retro_boomer_material.json'
LIMIT = 128 * 1024 * 1024


def policy():
    return json.loads(POLICY.read_text(encoding='utf-8'))


def digest(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def regular_file(path):
    path = Path(path)
    if path.is_symlink() or not path.is_file() or not 0 < path.stat().st_size <= LIMIT:
        raise ValueError(f'missing, empty, symlinked or oversized input: {path}')
    return path


def json_bytes(value):
    return (json.dumps(value, sort_keys=True, indent=2) + '\n').encode('ascii')


def bank_identity(p):
    return {'schema': 1, 'project': 'RETRO BOOMER', 'protocol': 'V1',
            'release_version': p['version'], 'approved_bank': p['bank'],
            'classes': ['Small', 'Large', 'Native']}


def pin_bytes(p):
    return f"RETRO_BOOMER_V1_RELEASE_BANK={p['bank']}\n".encode('ascii')


def reviewed_pin_file(path, p):
    content = regular_file(path).read_bytes()
    if not re.fullmatch(rb'RETRO_BOOMER_V1_RELEASE_BANK=[0-9a-f]{64}\n', content):
        raise ValueError('Expected exactly one native RETRO_BOOMER_V1_RELEASE_BANK assignment.')
    source_pin(p)
    if content != pin_bytes(p):
        raise ValueError('pin file differs from the approved native source bank')
    return p['bank']


def source_pin(p):
    if not isinstance(p.get('bank'), str) or not re.fullmatch('[0-9a-f]{64}', p['bank']):
        raise ValueError('Native RETRO BOOMER HistoryStep bank has not been approved.')
    source = (ROOT / 'noid_node/build.rs').read_text(encoding='utf-8')
    match = re.search(r'const RETRO_BOOMER_V1_APPROVED_BANK:.*?Some\(\[(.*?)\]\)', source, re.S)
    if not match or ''.join(re.findall(r'0x([0-9a-f]{2})', match[1])) != p['bank']:
        raise ValueError('release policy differs from the approved source bank')


def validate_raw(root):
    p = policy()
    source_pin(p)
    for name, expected in p['files'].items():
        if not isinstance(expected, str) or not re.fullmatch('[0-9a-f]{64}', expected):
            raise ValueError(f'native material has no approved SHA-256: {name}')
        if digest(regular_file(root / name)) != expected:
            raise ValueError(f'approved material SHA-256 mismatch: {name}')
    if (root / 'pins.env').read_bytes() != pin_bytes(p):
        raise ValueError('unexpected bank pin file')
    meta = (root / 'v21-runtime-metadata.bin').read_bytes()
    if len(meta) <= 108 or meta[:8] != b'O1V21BN1' or int.from_bytes(meta[104:108], 'little') != len(meta) - 108:
        raise ValueError('invalid runtime metadata framing')
    for index in range(3):
        b = (root / f'verified-block1-{index}.terminal').read_bytes()
        if len(b) < 74 or b[0] != 7 or int.from_bytes(b[1:9], 'little') != 1 or b[41] != index:
            raise ValueError('invalid block-one terminal framing')
    for name in p['files']:
        if name.endswith('.zst') and (root / name).read_bytes()[:4] != b'\x28\xb5\x2f\xfd':
            raise ValueError('invalid zstd framing')
    return p


def verify_manifest(root):
    p = validate_raw(root)
    expected = set(p['files']) | {'BANK.json', 'PROVENANCE.json', 'SHA256SUMS'}
    if root.is_symlink() or {f.name for f in root.iterdir()} != expected:
        raise ValueError('unexpected proof directory contents')
    contents = {n: regular_file(root / n).read_bytes() for n in expected}
    if contents['BANK.json'] != json_bytes(bank_identity(p)) or contents['PROVENANCE.json'] != json_bytes(p['provenance']):
        raise ValueError('bank identity or provenance differs from reviewed policy')
    manifest = ''.join(f'{hashlib.sha256(contents[n]).hexdigest()}  {n}\n' for n in sorted(expected - {'SHA256SUMS'}))
    if contents['SHA256SUMS'] != manifest.encode('ascii'):
        raise ValueError('invalid or incomplete proof manifest')
    return p


def create(args):
    root = args.pack
    p = validate_raw(root)
    if root.is_symlink() or {f.name for f in root.iterdir()} != set(p['files']):
        raise ValueError('raw bank must contain exactly the eight approved files')
    blobs = {n: (root / n).read_bytes() for n in p['files']}
    blobs['BANK.json'] = json_bytes(bank_identity(p))
    blobs['PROVENANCE.json'] = json_bytes(p['provenance'])
    blobs['SHA256SUMS'] = ''.join(f'{hashlib.sha256(blobs[n]).hexdigest()}  {n}\n' for n in sorted(blobs)).encode('ascii')
    args.output.mkdir(parents=True, exist_ok=True)
    output = args.output / p['archive_name']
    with output.open('xb') as raw, gzip.GzipFile(fileobj=raw, mode='wb', filename='', mtime=0) as gz:
        with tarfile.open(fileobj=gz, mode='w', format=tarfile.USTAR_FORMAT) as archive:
            for name in sorted(blobs):
                info = tarfile.TarInfo(p['archive_root'] + '/' + name)
                info.mode = 0o644
                info.size = len(blobs[name])
                archive.addfile(info, io.BytesIO(blobs[name]))
    print(f'{digest(output)}  {output.name}')


def review_members(members, p):
    required = {p['archive_root'] + '/' + n for n in (*p['files'], 'BANK.json', 'PROVENANCE.json', 'SHA256SUMS')}
    seen = set(); total = 0
    for member in members:
        if member.name in seen or member.name not in required or not member.isfile() or member.size <= 0:
            raise ValueError(f'unsafe or duplicate archive member: {member.name}')
        seen.add(member.name); total += member.size
        if total > LIMIT:
            raise ValueError('proof archive exceeds size limit')
    if seen != required:
        raise ValueError('incomplete proof archive')


def extract(args):
    p = policy()
    if not re.fullmatch('[0-9a-f]{64}', args.sha256) or digest(regular_file(args.archive)) != args.sha256:
        raise ValueError('proof archive SHA-256 mismatch')
    if p.get('archive_sha256') != args.sha256:
        raise ValueError('proof archive is not approved by the committed release policy')
    # The native archive root and extraction root must describe the same release.
    target = args.output / p['extraction_root']
    if args.output.is_symlink() or target.exists() or target.is_symlink():
        raise ValueError('unsafe or existing extraction destination')
    # Validate every member before writing; never call extractall or follow links.
    with tarfile.open(args.archive, 'r:gz') as archive:
        members = archive.getmembers()
        review_members(members, p)
        # All paths have exactly the reviewed root and one reviewed filename.
        target.mkdir(parents=True)
        for member in members:
            with archive.extractfile(member) as source, (target / member.name.split('/')[1]).open('xb') as dest:
                while chunk := source.read(1024 * 1024):
                    dest.write(chunk)
    verify_manifest(target)
    print(f'Verified transport, approved hashes and framing: {target}')
    print('Semantic matrix authentication remains mandatory in noid_node/build.rs; terminals were not reverified.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest='command', required=True)
    c = subs.add_parser('create'); c.add_argument('--pack', type=Path, required=True); c.add_argument('--output', type=Path, required=True)
    e = subs.add_parser('extract'); e.add_argument('--archive', type=Path, required=True); e.add_argument('--sha256', required=True); e.add_argument('--output', type=Path, required=True)
    v = subs.add_parser('verify'); v.add_argument('--root', type=Path, required=True)
    r = subs.add_parser('raw'); r.add_argument('--root', type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == 'create': create(args)
        elif args.command == 'extract': extract(args)
        elif args.command == 'verify': verify_manifest(args.root); print('Approved proof files, manifest and framing verified; no proof verification performed.')
        else: validate_raw(args.root); print('Approved raw proof files and framing verified; no proof verification performed.')
    except (OSError, ValueError, UnicodeError, KeyError, tarfile.TarError) as error:
        parser.exit(1, f'release material: {error}\n')


if __name__ == '__main__':
    main()
