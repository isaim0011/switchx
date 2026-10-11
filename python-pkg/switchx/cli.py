import sys
import subprocess
import shutil

def main():
    binary = shutil.which("switchx") or shutil.which("switchx.exe")
    if not binary:
        print("\033[96mSwitchX Native Core not found on PATH.\033[0m")
        print("To install the high-speed compiled Rust engine, run:\n")
        print("  \033[92mcargo install switchx\033[0m\n")
        print("Repository: https://github.com/isaim0011/switchx")
        sys.exit(1)
    
    args = [binary] + sys.argv[1:]
    res = subprocess.run(args)
    sys.exit(res.returncode)

if __name__ == "__main__":
    main()
