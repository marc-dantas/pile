import argparse
import subprocess
import sys
import json
from pathlib import Path

def log(t: str):
    print(f"{t}", file=sys.stderr)

def msg_fail(t: str):
    print(f"FAIL: {t}", file=sys.stderr)

def msg_pass(t: str):
    print(f"PASS: {t}", file=sys.stderr)

def warn(t: str):
    print(f"warning: {t}", file=sys.stderr)

def err(t: str):
    print(f"error: {t}", file=sys.stderr)


EXAMPLES_DIR = Path('./examples')
TEST_DIR = EXAMPLES_DIR / 'test'


def run_pile_program(filepath: Path):
    try:
        result = subprocess.run(
            ["cargo", "run", "--quiet", "--", str(filepath)],
            capture_output=True,
            text=True
        )
        return result.stdout, result.returncode
    except Exception as e:
        err(f"failed to execute {filepath}: {e}")
        return None, None


def write_mode(missing_files=None):
    TEST_DIR.mkdir(exist_ok=True)
    files_to_process = missing_files if missing_files else EXAMPLES_DIR.glob("*.pile")

    for file in files_to_process:
        stdout, returncode = run_pile_program(file)
        
        if stdout is not None:
            data = {
                "file": str(file),
                "stdout": stdout,
                "returncode": returncode
            }

            output_path = TEST_DIR / f"{file.stem}.json"
            with output_path.open("w") as f:
                json.dump(data, f, indent=4)
            
            log(f"test written for {file.name} into {f'{file.stem}.json'}")

    log("\nTEST WRITE COMPLETED")


def test_mode():
    fail_count = 0
    success_count = 0
    missing_files = []
    
    for file in EXAMPLES_DIR.glob("*.pile"):
        stdout, returncode = run_pile_program(file)
        
        if stdout is not None:
            result_path = TEST_DIR / f"{file.stem}.json"
            if not result_path.exists():
                warn(f"no result file found for {file.name}, marking as missing.")
                missing_files.append(file)
                continue

            with result_path.open("r") as f:
                xs = json.load(f)

            try:
                assert stdout == xs["stdout"]
                assert returncode == xs["returncode"]
            except AssertionError:
                fail_count += 1
                msg_fail(f"test file {f'{file.stem}.json'} for {file.name}")
                continue
                
            success_count += 1
            msg_pass(f"test file {f'{file.stem}.json'} for {file.name}")

    log("\nTEST COMPLETED")
    log(f"{fail_count} FAILED")
    log(f"{success_count} PASSED")
    
    if missing_files:
        if input(f"\n{len(missing_files)} test files are missing. Create them now? (y/n): ").lower() == 'y':
            write_mode(missing_files=missing_files)
            log("\nMISSING TEST FILES CREATED.")
        else:
            warn("test incomplete due to missing files. Run in 'write' mode to create all test files.")


def main():
    parser = argparse.ArgumentParser(description="CLI for testing Pile language programs.")
    parser.add_argument("mode", choices=["write", "test"], help="Mode of operation: write or test")

    args = parser.parse_args()
    if args.mode == "write":
        write_mode()
    elif args.mode == "test":
        test_mode()


if __name__ == "__main__":
    main()
