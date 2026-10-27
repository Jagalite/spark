#!/usr/bin/env python3
"""Bounded regression tests for capture provenance; no terminal is opened."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('capture_terminal', Path(__file__).with_name('capture_terminal.py'))
capture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)


def artifact(path, name='spark-gallery'):
    return json.dumps({'reason': 'compiler-artifact', 'target': {'name': name, 'kind': ['bin']}, 'executable': str(path)})


class CaptureProvenanceTests(unittest.TestCase):
    def test_output_preserves_glyphs_split_across_reads(self):
        encoded = 'start ◆界✦ end'.encode('utf-8')
        for split in range(len(encoded) + 1):
            with self.subTest(split=split):
                events = []
                recorder = capture.OutputRecorder(events, 0)
                recorder.record(encoded[:split])
                recorder.record(encoded[split:])
                recorder.record(b'', final=True)
                self.assertEqual(''.join(event[2] for event in events), 'start ◆界✦ end')
                self.assertTrue(all(event[1] == 'o' and event[2] for event in events))

    def test_output_flush_handles_incomplete_utf8_without_empty_events(self):
        events = []
        recorder = capture.OutputRecorder(events, 0)
        recorder.record(b'\xe2')
        self.assertEqual(events, [])
        recorder.record(b'', final=True)
        self.assertEqual([event[2] for event in events], ['\ufffd'])
        empty_events = []
        capture.OutputRecorder(empty_events, 0).record(b'', final=True)
        self.assertEqual(empty_events, [])

    def test_cargo_artifact_path_not_default_target_path(self):
        path = Path('/custom/build/debug/spark-gallery')
        messages = artifact('/unrelated', 'other') + '\n' + artifact(path)
        self.assertEqual(capture.gallery_executable(messages), path)

    def test_missing_or_ambiguous_artifact_is_rejected(self):
        for messages in ['', artifact('/one') + '\n' + artifact('/two')]:
            with self.subTest(messages=messages), self.assertRaises(RuntimeError):
                capture.gallery_executable(messages)

    def test_failed_render_never_accepts_old_stamp(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'previews').mkdir()
            (root / 'previews/segmented-comet.cells.json').write_text(json.dumps({'source_sha256': 'a' * 64}))
            with patch.object(capture.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'cargo')):
                with self.assertRaises(subprocess.CalledProcessError):
                    capture.fresh_source_hash(root)

    def test_render_replaces_stale_source_stamp(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'previews').mkdir()
            target = root / 'previews/segmented-comet.cells.json'
            target.write_text(json.dumps({'source_sha256': 'a' * 64}))
            def run(command, **kwargs):
                self.assertEqual(command[-2:], ['render', 'segmented-comet'])
                self.assertTrue(kwargs['check'])
                target.write_text(json.dumps({'source_sha256': 'b' * 64}))
            with patch.object(capture.subprocess, 'run', side_effect=run):
                self.assertEqual(capture.fresh_source_hash(root), 'b' * 64)

    def test_source_change_during_build_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / 'custom-gallery'
            binary.write_bytes(b'gallery')
            build = subprocess.CompletedProcess([], 0, stdout=artifact(binary))
            with patch.object(capture, 'fresh_source_hash', side_effect=['a' * 64, 'b' * 64]), patch.object(capture.subprocess, 'run', return_value=build):
                with self.assertRaisesRegex(RuntimeError, 'source changed'):
                    capture.prepare_capture(root)

    def test_binary_change_during_recording_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / 'gallery'
            binary.write_bytes(b'changed')
            digest = hashlib.sha256(b'original').hexdigest()
            with patch.object(capture, 'fresh_source_hash', return_value='a' * 64):
                with self.assertRaisesRegex(RuntimeError, 'binary changed'):
                    capture.verify_capture_inputs(root, binary, 'a' * 64, digest)

    def test_preparation_returns_current_binary_and_source(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / 'gallery'
            binary.write_bytes(b'gallery')
            build = subprocess.CompletedProcess([], 0, stdout=artifact(binary))
            with patch.object(capture, 'fresh_source_hash', return_value='b' * 64), patch.object(capture.subprocess, 'run', return_value=build) as run:
                self.assertEqual(capture.prepare_capture(root), (binary, 'b' * 64, hashlib.sha256(b'gallery').hexdigest()))
                self.assertIn('--locked', run.call_args.args[0])


if __name__ == '__main__':
    unittest.main()
