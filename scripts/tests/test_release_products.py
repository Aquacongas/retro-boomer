"""Small release policy and architecture tests; never build or run a daemon."""
from pathlib import Path
import argparse
import contextlib
import io
import json
import os
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts/release'))
import release_products as products
import workflow_gate as gate


class ReleaseProductTests(unittest.TestCase):
    def test_candidate_uses_release_notes_for_its_version(self):
        policy = products.material.policy()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            inputs = root / 'inputs'; inputs.mkdir()
            for platform in products.PLATFORMS:
                for name in (*products.names(platform, policy['version']), f'BUILD-{platform}.json'):
                    (inputs / name).write_bytes(b'packaging fixture')
            pack = root / policy['archive_name']; pack.write_bytes(b'approved pack fixture')
            args = argparse.Namespace(inputs=inputs, pack=pack, output=root/'candidate',
                                      source_commit='a'*40, run_id='123')
            with patch.object(products.material, 'policy', return_value={**policy, 'archive_sha256': products.material.digest(pack)}), \
                 patch.object(products, 'verify_candidate'), contextlib.redirect_stdout(io.StringIO()):
                products.assemble(args)
            notes = (args.output/'RELEASE_NOTES.md').read_bytes()
            self.assertEqual(notes, (ROOT/'.github/release-notes'/f'v{policy["version"]}.md').read_bytes())
            manifest = json.loads((args.output/'candidate.json').read_text())
            self.assertEqual(manifest['files']['RELEASE_NOTES.md'], products.material.digest(args.output/'RELEASE_NOTES.md'))

    def test_media_is_recorded_and_packaged_only_in_linux_x86_64_core(self):
        policy = {**products.material.policy(), 'version': '1.0.1'}
        smoke = {'genesis': products.GENESIS, 'isolated': True,
                 'native_runtime': 'ready', 'activation_height': 1,
                 'mining': False, 'issuance_units': '10000000000000000'}
        for platform in products.PLATFORMS:
            with self.subTest(platform=platform), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                bin_dir = root / 'bin'; bin_dir.mkdir()
                suffix = '.exe' if platform.startswith('windows-') else ''
                header = bytearray(128)
                if platform.startswith('linux-'):
                    header[:6] = b'\x7fELF\x02\x01'
                    header[18:20] = (62 if platform.endswith('x86_64') else 183).to_bytes(2, 'little')
                elif platform.startswith('windows-'):
                    header[:2] = b'MZ'; header[60:64] = (64).to_bytes(4, 'little')
                    header[64:70] = b'PE\x00\x00\x64\x86'; header[88:90] = b'\x0b\x02'
                else:
                    header[:4] = b'\xcf\xfa\xed\xfe'
                    header[4:8] = (0x01000007 if platform.endswith('x86_64') else 0x0100000c).to_bytes(4, 'little')
                for name in ('retro-boomerd', 'retro-boomer-cli', 'retro-boomer-miner', 'retro-boomer-gui', 'retro-boomer-media'):
                    (bin_dir / (name + suffix)).write_bytes(header)
                smoke_path = root / 'smoke.json'; smoke_path.write_text(json.dumps(smoke))
                record_path = root / 'BUILD.json'
                args = argparse.Namespace(bin_dir=bin_dir, platform=platform, target_dir=root,
                                          smoke=smoke_path, record=record_path, output=record_path)
                with patch.object(products.material, 'policy', return_value=policy), \
                     patch.object(products, 'matrix_record', return_value={}):
                    products.record(args)
                    record = json.loads(record_path.read_text())
                    self.assertEqual('retro-boomer-media' in record['binaries'], platform == 'linux-x86_64')
                    self.assertNotIn('retro-boomer-media.exe', record['binaries'])
                    args.output = root / 'products'
                    with contextlib.redirect_stdout(io.StringIO()):
                        products.core(args)
                    archive_path = args.output / products.names(platform, '1.0.1')[0]
                    if platform.startswith('windows-'):
                        with zipfile.ZipFile(archive_path) as archive:
                            members = {Path(name).name for name in archive.namelist()}
                    else:
                        with tarfile.open(archive_path) as archive:
                            members = {Path(item.name).name for item in archive}
                            if platform == 'linux-x86_64':
                                media = next(item for item in archive.getmembers() if item.name.endswith('/retro-boomer-media'))
                                self.assertEqual(media.mode, 0o755)
                    expected = {name + suffix for name in ('retro-boomerd', 'retro-boomer-cli', 'retro-boomer-miner')}
                    if platform == 'linux-x86_64':
                        expected.add('retro-boomer-media')
                        with self.assertRaisesRegex(ValueError, 'Core payload hash mismatch'):
                            products.verify_core(archive_path, platform,
                                                 {**record, 'binaries': {**record['binaries'], 'retro-boomer-media': '0'*64}})
                        (bin_dir / 'retro-boomer-media').unlink()
                        with self.assertRaises(ValueError):
                            products.record(args)
                    self.assertEqual(members, expected | set(products.document_sources(platform)) | {'BUILD.json'})

    def test_linux_service_is_an_explicit_optional_payload(self):
        for platform in products.PLATFORMS:
            documents = products.document_sources(platform)
            self.assertEqual('retro-boomerd.service' in documents, platform.startswith('linux-'))
            self.assertTrue(all(path.is_file() for path in documents.values()))
        unit = products.document_sources('linux-x86_64')['retro-boomerd.service'].read_text()
        self.assertIn('ExecStart=/usr/lib/retro-boomer/retro-boomerd --mode node', unit)
        self.assertNotIn('--mode miner', unit)

    def test_exact_ten_products_and_one_bank(self):
        p=products.material.policy(); names=products.expected_assets(p)
        self.assertEqual(len(names),11)
        self.assertEqual(sum(n.startswith('retro-boomer-core-') for n in names),5)
        self.assertEqual(sum(n.startswith('retro-boomer-gui-') for n in names),5)
        self.assertFalse(any('parano1d' in n or 'retro-boomer-media' in n for n in names))

    @unittest.skipUnless(sys.platform.startswith('linux'),'inspect the real running Linux interpreter')
    def test_real_elf_architecture_and_cross_format_rejection(self):
        import platform
        arch={'x86_64':'linux-x86_64','aarch64':'linux-aarch64'}.get(platform.machine())
        if arch is None: self.skipTest('unsupported interpreter architecture')
        products.architecture(Path(sys.executable).resolve(),arch)
        with self.assertRaises(ValueError): products.architecture(Path(sys.executable).resolve(),'windows-x86_64')

    def test_incomplete_candidate_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises((OSError,ValueError)): products.verify_candidate(Path(directory))

    def test_smoke_report_checks_all_recorded_release_parameters(self):
        expected = {'genesis': products.GENESIS, 'isolated': True,
                    'native_runtime': 'ready', 'activation_height': 1,
                    'mining': False, 'issuance_units': '10000000000000000'}
        products.validate_smoke_report(expected)
        for key, wrong in [('genesis', '0'*64), ('isolated', False),
                           ('native_runtime', 'unavailable'), ('activation_height', 2),
                           ('mining', True), ('issuance_units', '210000000000000')]:
            with self.assertRaises(ValueError):
                products.validate_smoke_report({**expected, key: wrong})
        with self.assertRaises(ValueError):
            products.validate_smoke_report({})

    def test_workflows_do_not_publish_from_ci_or_candidate(self):
        for name in ['platform-ci.yml','release.yml']:
            text=(ROOT/'.github/workflows'/name).read_text()
            self.assertNotIn('contents: write',text)
            self.assertNotIn('gh release create',text)
            self.assertNotIn('git push',text)
        publisher=(ROOT/'.github/workflows/publish.yml').read_text()
        self.assertEqual(publisher.count('contents: write'),1)
        self.assertIn('environment: retro-boomer-release',publisher)
        self.assertNotIn('cargo ',publisher)
        self.assertNotIn('\n  push:',publisher)
        self.assertNotIn('\n  pull_request:',publisher)
        self.assertIn('--verify-tag',publisher)

    def test_workflow_gate_rejects_foreign_context_without_network(self):
        old=os.environ.get('GITHUB_REPOSITORY')
        try:
            os.environ['GITHUB_REPOSITORY']='untrusted/example'
            with self.assertRaises(ValueError): gate.context()
        finally:
            if old is None: os.environ.pop('GITHUB_REPOSITORY',None)
            else: os.environ['GITHUB_REPOSITORY']=old

    def test_user_inputs_are_not_shell_fragments(self):
        name='RETRO_BOOMER_RELEASE_TEST_INPUT'; old=os.environ.get(name)
        try:
            for value in ['0','../123','1; touch forbidden','$(echo unsafe)','', '-1']:
                os.environ[name]=value
                with self.assertRaises(ValueError): gate.number(name)
            os.environ[name]='12345'; self.assertEqual(gate.number(name),'12345')
            for value in ['a'*63,'A'*64,'https://example.invalid']:
                os.environ[name]=value
                with self.assertRaises(ValueError): gate.sha(name)
        finally:
            if old is None: os.environ.pop(name,None)
            else: os.environ[name]=old


if __name__=='__main__': unittest.main()
