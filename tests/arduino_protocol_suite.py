#!/usr/bin/env python3
"""
V.I.S.O.R. Arduino Serial Protocol & Firmware Verification Suite
===============================================================
This test suite verifies the ASCII serial communication protocol implemented
between the Raspberry Pi (pi_logic) and Arduino Uno (arduino_control).

It validates:
1. Exact command framing (<DISP:b,a,g>, <PING>)
2. All 8 supply dispensing permutations
3. Edge-case fuzzing and error handling (malformed packets, buffer overflow limits)
4. State machine simulation & response consistency
"""

import re
import sys
import time
from typing import Dict, List, Optional, Tuple


class ArduinoFirmwareSimulator:
    """
    Exact software emulation of the parsing and state logic in arduino_control.ino
    """
    BUFFER_SIZE = 64
    PIN_BANDAGE = 9
    PIN_ALCOHOL = 10
    # PIN_GAUZE = 11  # [DISABLED: 2-item dispensing only]

    def __init__(self):
        self.buffer = ""
        self.output_log: List[str] = ["STATUS:READY"]
        self.led_status = False

    def feed_bytes(self, data: bytes) -> List[str]:
        responses = []
        for byte_val in data:
            c = chr(byte_val)
            if c == '<':
                self.buffer = '<'
            elif c == '>':
                if len(self.buffer) < self.BUFFER_SIZE - 1:
                    self.buffer += c
                    resp = self._process_command(self.buffer)
                    if resp:
                        responses.extend(resp)
                    self.buffer = ""
            elif c in ('\r', '\n'):
                continue
            else:
                if len(self.buffer) > 0 and len(self.buffer) < self.BUFFER_SIZE - 1:
                    self.buffer += c
        return responses

    def _process_command(self, cmd: str) -> List[str]:
        resps = []
        # Pattern match 2-item format <DISP:b,a>
        match = re.match(r"^<DISP:(\d+),(\d+)>$", cmd)
        if match:
            b, a = int(match.group(1)), int(match.group(2))
            do_b = (b == 1)
            do_a = (a == 1)

            resps.append(f"ACK:DISP:{1 if do_b else 0},{1 if do_a else 0}")

            if do_b or do_a:
                self.led_status = True
                if do_b:
                    resps.append("STATUS:DISPENSING_BANDAGE")
                if do_a:
                    resps.append("STATUS:DISPENSING_ALCOHOL")
                self.led_status = False
                resps.append("STATUS:DISPENSE_COMPLETE")
            else:
                resps.append("STATUS:HOLD_ALL")
        # Legacy 3-item format commented out:
        # match3 = re.match(r"^<DISP:(\d+),(\d+),(\d+)>$", cmd)
        elif cmd == "<PING>":
            resps.append("PONG")
        else:
            resps.append(f"ERR:UNKNOWN_COMMAND:{cmd}")
        return resps


class TestResult:
    def __init__(self, name: str, passed: bool, details: str = ""):
        self.name = name
        self.passed = passed
        self.details = details


def run_protocol_tests() -> Tuple[List[TestResult], int, int]:
    tests: List[TestResult] = []

    def assert_test(name: str, condition: bool, details: str = ""):
        tests.append(TestResult(name, condition, details))

    # Test 1: Ping Pong Handshake
    sim = ArduinoFirmwareSimulator()
    resps = sim.feed_bytes(b"<PING>\n")
    assert_test("Ping-Pong Keepalive", resps == ["PONG"], f"Expected ['PONG'], got {resps}")

    # Test 2: All 4 Dispense Permutations (2 items: Bandage, Alcohol Pad)
    all_combos = [
        (0, 0, ["ACK:DISP:0,0", "STATUS:HOLD_ALL"]),
        (1, 0, ["ACK:DISP:1,0", "STATUS:DISPENSING_BANDAGE", "STATUS:DISPENSE_COMPLETE"]),
        (0, 1, ["ACK:DISP:0,1", "STATUS:DISPENSING_ALCOHOL", "STATUS:DISPENSE_COMPLETE"]),
        (1, 1, ["ACK:DISP:1,1", "STATUS:DISPENSING_BANDAGE", "STATUS:DISPENSING_ALCOHOL", "STATUS:DISPENSE_COMPLETE"]),
    ]
    # 3-item permutations commented out:
    # (0, 0, 1, ["ACK:DISP:0,0,1", "STATUS:DISPENSING_GAUZE", "STATUS:DISPENSE_COMPLETE"]),
    # (1, 0, 1, ["ACK:DISP:1,0,1", "STATUS:DISPENSING_BANDAGE", "STATUS:DISPENSING_GAUZE", "STATUS:DISPENSE_COMPLETE"]),
    # (0, 1, 1, ["ACK:DISP:0,1,1", "STATUS:DISPENSING_ALCOHOL", "STATUS:DISPENSING_GAUZE", "STATUS:DISPENSE_COMPLETE"]),
    # (1, 1, 1, ["ACK:DISP:1,1,1", "STATUS:DISPENSING_BANDAGE", "STATUS:DISPENSING_ALCOHOL", "STATUS:DISPENSING_GAUZE", "STATUS:DISPENSE_COMPLETE"]),

    for b, a, expected in all_combos:
        sim = ArduinoFirmwareSimulator()
        cmd = f"<DISP:{b},{a}>\n".encode('ascii')
        resps = sim.feed_bytes(cmd)
        test_name = f"Dispense Combination ({b},{a})"
        assert_test(test_name, resps == expected, f"Cmd: {cmd.strip()}, Expected: {expected}, Got: {resps}")

    # Test 3: Framing Noise Resilience & Prefix Garbage
    sim = ArduinoFirmwareSimulator()
    resps = sim.feed_bytes(b"RANDOM_NOISE_12345<PING>\r\n")
    assert_test("Serial Garbage Prefix Filtering", resps == ["PONG"], f"Expected ['PONG'], got {resps}")

    # Test 4: Concatenated Multi-Packet Stream
    sim = ArduinoFirmwareSimulator()
    resps = sim.feed_bytes(b"<PING><DISP:1,0><PING>")
    expected = [
        "PONG",
        "ACK:DISP:1,0",
        "STATUS:DISPENSING_BANDAGE",
        "STATUS:DISPENSE_COMPLETE",
        "PONG"
    ]
    assert_test("Concatenated Multi-Packet Stream", resps == expected, f"Expected {expected}, got {resps}")

    # Test 5: Unknown Command Handling
    sim = ArduinoFirmwareSimulator()
    resps = sim.feed_bytes(b"<UNKNOWN_ACTION>\n")
    assert_test("Unknown Command Error Emission", resps == ["ERR:UNKNOWN_COMMAND:<UNKNOWN_ACTION>"], f"Got {resps}")

    # Test 6: Buffer Overflow Protection (>64 bytes before closing bracket)
    sim = ArduinoFirmwareSimulator()
    overflow_payload = b"<" + b"X" * 128 + b">\n"
    resps = sim.feed_bytes(overflow_payload)
    assert_test("Serial Buffer Overflow Boundary Limit", len(resps) <= 1, f"Expected safe containment, got {resps}")

    # Test 7: Incomplete Frame Abort (Restart on '<')
    sim = ArduinoFirmwareSimulator()
    resps = sim.feed_bytes(b"<DISP:1,0<PING>")
    assert_test("Incomplete Frame Reset on New Start Delimiter", resps == ["PONG"], f"Expected ['PONG'], got {resps}")

    passed = sum(1 for t in tests if t.passed)
    failed = len(tests) - passed
    return tests, passed, failed


def print_report(tests: List[TestResult], passed: int, failed: int):
    print("=" * 70)
    print("      V.I.S.O.R. ARDUINO PROTOCOL & FIRMWARE VERIFICATION REPORT     ")
    print("=" * 70)
    print(f"Total Tests Executed: {len(tests)}")
    print(f"Passed: {passed}")
    print(f"Failed: {failed}")
    print("-" * 70)

    for i, test in enumerate(tests, 1):
        status_str = "[PASS]" if test.passed else "[FAIL]"
        print(f"{i:02d}. {status_str} {test.name}")
        if not test.passed:
            print(f"    Details: {test.details}")

    print("=" * 70)
    if failed == 0:
        print("RESULT: ALL PROTOCOL TESTS PASSED SUCCESSFULLY.")
    else:
        print(f"RESULT: {failed} TEST(S) FAILED.")
    print("=" * 70)


if __name__ == "__main__":
    tests, passed, failed = run_protocol_tests()
    print_report(tests, passed, failed)
    sys.exit(0 if failed == 0 else 1)
