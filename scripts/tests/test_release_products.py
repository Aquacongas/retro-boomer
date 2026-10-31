"""Small release policy and architecture tests; never build or run a daemon."""
from pathlib import Path
import os
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts/release'))
import release_products as products
import workflow_gate as gate


class ReleaseProductTests(unittest.TestCase):
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
