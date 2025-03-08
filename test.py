import argparse
import subprocess
import sys
import json
from pathlib import Path

DEFAULT_STDIN = "DEFAULT_STDIN"

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
            ["cargo", "run", "--", str(filepath)],
            input=DEFAULT_STDIN,
            capture_output=True,
            text=True
        )
        return result.stdout, result.returncode
    except subprocess.TimeoutExpired:
        err(f"execution of {filepath} timed out")
        return None, None
    except Exception as e:
        err(f"failed to execute {filepath}: {e}")
        return None, None


def write_mode(missing_files=None):
    log("WRITE MODE")
    
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


def show_failed_test(name: str, t: tuple[tuple[str, str], tuple[str, str]]):
    print(f"FAILED TEST {name}", file=sys.stderr)
    print("stdout:", file=sys.stderr)
    a = t[0][1].replace('\n', "\\n")
    b = t[0][0].replace('\n', "\\n")
    print(f"  expected: {a}", file=sys.stderr)
    print(f"  got: {b}", file=sys.stderr)
    print(f"returncode:", file=sys.stderr)
    print(f"  expected: {t[1][1]}", file=sys.stderr)
    print(f"  got: {t[1][0]}", file=sys.stderr)
    print()


def test_mode():
    log("TEST MODE")
    
    fails = 0
    success_count = 0
    missing_files = []
    
    
    log(f"TESTING FILES IN DIRECTORY: {EXAMPLES_DIR}\n")
    
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
                fails += 1
                msg_fail(f"test file {f'{file.stem}.json'} for {file.name}")
                show_failed_test(name=result_path, t=((stdout, xs["stdout"]), (returncode, xs["returncode"])))
                continue
        
            success_count += 1

    total = fails + success_count 

    log("TEST COMPLETED")
    log(f"{fails}/{total} FAILED")
    log(f"{success_count}/{total} PASSED")
    
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
