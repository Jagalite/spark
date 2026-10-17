#!/usr/bin/env python3
"""Focused tests for post-measurement harness review repairs; no host rerun."""
import importlib.util
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('current_harness', ROOT/'scripts/qualify_host_endurance.py')
harness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(harness)

class WindowFixture:
    name = 'paced_host'
    def __init__(self, output):
        self.bytes_output = 0
        self.errors, self.windows = [], []
        self.readings = iter([0, 1.024])
        self.pumps = 0
        self.output = output
    def pump(self, duration):
        self.pumps += 1
        if self.pumps == 2:
            self.bytes_output += self.output
    def sample(self, force):
        return {'cpu_ticks': 0}
    def elapsed(self):
        return next(self.readings)
    def note(self, message):
        pass

for label in ['active', 'resumed']:
    fixture = WindowFixture(0)
    harness.HostRun.window(fixture, label)
    assert fixture.errors == [f'{label}: active/resumed output liveness check failed']
    fixture = WindowFixture(19)
    harness.HostRun.window(fixture, label)
    assert fixture.errors == []
for mode, expected in [(0, 'a  '), (1, '  c'), (2, '   ')]:
    screen = harness.StatusScreen()
    screen.feed('abc\x1b[1;2H'+f'\x1b[{mode}J')
    assert screen.row(0)[:3] == expected, (mode, screen.row(0))
result = {'status':'passed', 'scope':'Post-measurement harness guards, not a rerun of host qualification',
          'cases':['active zero output rejects','resumed zero output rejects',
                   'active positive output accepts','resumed positive output accepts',
                   'CSI 0J excludes cursor onward','CSI 1J excludes beginning through cursor',
                   'CSI 2J clears screen']}
(Path(__file__).parent/'harness-guard-tests.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
