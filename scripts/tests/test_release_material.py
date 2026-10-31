#!/usr/bin/env python3
"""Test release transport using real approved material, never dummy proofs."""
import argparse
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
import release_material as material


class ReleaseMaterialTests(unittest.TestCase):
    def test_unapproved_native_bank_blocks_production(self):
        p = material.policy()
        if p['bank'] is None:
            with self.assertRaisesRegex(ValueError, 'has not been approved'):
                material.source_pin(p)
            self.assertIsNone(p['archive_sha256'])
            self.assertTrue(all(value is None for value in p['files'].values()))
        else:
            material.source_pin(p)
            self.assertRegex(p['bank'], r'^[0-9a-f]{64}$')
        self.assertEqual(len(p['files']), 8)
        self.assertEqual(p['archive_root'], p['extraction_root'])

    def test_archive_members_reject_traversal_links_duplicates_and_missing_files(self):
        p=material.policy()
        names=[p['archive_root']+'/'+n for n in (*p['files'],'BANK.json','PROVENANCE.json','SHA256SUMS')]
        def members():
            result=[]
            for name in names:
                info=tarfile.TarInfo(name); info.size=1; result.append(info)
            return result
        material.review_members(members(),p)
        for bad in ['../outside',p['archive_root']+'/../outside',p['archive_root']+'/extra']:
            items=members(); items[0].name=bad
            with self.assertRaises(ValueError): material.review_members(items,p)
        for kind in [tarfile.SYMTYPE,tarfile.LNKTYPE,tarfile.DIRTYPE]:
            items=members(); items[0].type=kind
            with self.assertRaises(ValueError): material.review_members(items,p)
        items=members()
        with self.assertRaises(ValueError): material.review_members(items+[items[0]],p)
        with self.assertRaises(ValueError): material.review_members(items[:-1],p)
        items=members(); items[0].size=material.LIMIT+1
        with self.assertRaises(ValueError): material.review_members(items,p)

    def test_wrong_archive_digest_fails_before_extraction(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); archive=root/'invalid.tar.gz'; archive.write_bytes(b'not an approved archive')
            with self.assertRaisesRegex(ValueError,'SHA-256'):
                material.extract(argparse.Namespace(archive=archive,sha256='0'*64,output=root/'out'))
            self.assertFalse((root/'out').exists())

    def test_symlinked_input_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); target=root/'public.txt'; target.write_text('not proof material')
            try: (root/'link').symlink_to(target)
            except OSError: self.skipTest('symlink creation unavailable')
            with self.assertRaises(ValueError): material.regular_file(root/'link')

    @unittest.skipUnless(material.policy().get('archive_sha256') and (ROOT/'dist/materials/retro-boomer-history-step-pack-v1.0.0.tar.gz').is_file(), 'no approved native archive is available')
    def test_existing_archive_extracts_and_detects_manifest_tampering(self):
        with tempfile.TemporaryDirectory() as directory, contextlib.redirect_stdout(io.StringIO()):
            root=Path(directory)
            p=material.policy(); archive=ROOT/'dist/materials'/p['archive_name']
            self.assertEqual(material.digest(archive),p['archive_sha256'])
            material.extract(argparse.Namespace(archive=archive,sha256=p['archive_sha256'],output=root/'unpack'))
            unpacked=root/'unpack'/p['extraction_root']
            material.verify_manifest(unpacked)
            (unpacked/'BANK.json').write_text('{}\n')
            with self.assertRaises(ValueError): material.verify_manifest(unpacked)
            with self.assertRaises(ValueError):
                material.extract(argparse.Namespace(archive=archive,sha256=p['archive_sha256'],output=root/'unpack'))


if __name__=='__main__': unittest.main()
