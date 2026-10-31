"""Negative TAPE input tests; legacy filename retained for test discovery."""
from pathlib import Path
import json
import os
import subprocess
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
BUILD=ROOT/'scripts/build_release.sh'
BANK=json.loads((ROOT/'scripts/release/retro_boomer_material.json').read_text())['bank'] or '0'*64


@unittest.skipUnless(os.name=='posix','Bash preflight is exercised by Linux tooling CI')
class RetroBoomerV1ReleaseInputs(unittest.TestCase):
    def run_script(self,script,*args):
        return subprocess.run(['bash',str(script),*map(str,args)],cwd=ROOT,text=True,capture_output=True,timeout=10)

    def test_missing_material_fails_before_cargo(self):
        result=self.run_script(BUILD)
        self.assertEqual(result.returncode,2)
        self.assertIn('real V1 pack',result.stderr)

    def test_historical_arguments_are_not_accepted(self):
        result=self.run_script(BUILD,'--v2-pack','/not-a-pack')
        self.assertEqual(result.returncode,2)
        self.assertIn('--v1-pack',result.stderr)

    def test_bad_pins_fail_before_output_or_cargo(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); pins=root/'pins.env'; output=root/'out'
            for content in ['NOID_V2_RELEASE_BANK='+BANK+'\n',
                            'RETRO_BOOMER_V1_RELEASE_BANK='+BANK+'\nRETRO_BOOMER_V1_RELEASE_BANK='+BANK+'\n',
                            'RETRO_BOOMER_V1_RELEASE_BANK=$(touch forbidden)\n']:
                pins.write_text(content)
                result=self.run_script(BUILD,'--v1-pack',root,'--v1-pins',pins,'--output',output)
                self.assertNotEqual(result.returncode,0)
                self.assertIn('exactly one',result.stderr)
                self.assertFalse(output.exists())
                self.assertFalse((ROOT/'forbidden').exists())

    def test_approved_pin_does_not_replace_missing_matrices(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); pins=root/'pins.env'; pins.write_text('RETRO_BOOMER_V1_RELEASE_BANK='+BANK+'\n')
            result=self.run_script(BUILD,'--v1-pack',root,'--v1-pins',pins,'--output',root/'out')
            self.assertNotEqual(result.returncode,0)
            self.assertTrue('release material:' in result.stderr or 'has not been approved' in result.stderr)
            self.assertFalse((root/'out').exists())

    def test_symlinked_pin_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); actual=root/'actual.env'; actual.write_text('RETRO_BOOMER_V1_RELEASE_BANK='+BANK+'\n'); (root/'pins.env').symlink_to(actual)
            result=self.run_script(BUILD,'--v1-pack',root,'--v1-pins',root/'pins.env','--output',root/'out')
            self.assertEqual(result.returncode,2)
            self.assertIn('Invalid bank directory or pin file',result.stderr)

    def test_legacy_publisher_stays_disabled(self):
        result=self.run_script(ROOT/'scripts/publish_release.sh','1.0.0','--force')
        self.assertEqual(result.returncode,2)
        self.assertIn('publication is disabled',result.stderr)


if __name__=='__main__': unittest.main()
