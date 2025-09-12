import os
import shutil

BASE_DIR = os.path.join(os.path.dirname(__file__), "../python/potato_head")
FOLDERS = [
    "logging",
    "mock",
    "google",
    "openai",
]


def copy_pyi(folder_path, name):
    src = os.path.join(folder_path, "__init__.pyi")
    dst = os.path.join(folder_path, f"_{name}.pyi")
    if os.path.exists(src):
        shutil.copyfile(src, dst)
        print(f"Copied {src} -> {dst}")
    else:
        print(f"Skipped {name}: {src} does not exist")


def process_folder(folder):
    folder_path = os.path.join(BASE_DIR, folder)
    copy_pyi(folder_path, os.path.basename(folder))


def copy_base_init():
    src = os.path.join(BASE_DIR, "__init__.pyi")
    dst = os.path.join(BASE_DIR, "_potato_head.pyi")
    if os.path.exists(src):
        shutil.copyfile(src, dst)
        print(f"Copied {src} -> {dst}")
    else:
        print(f"Skipped base: {src} does not exist")


copy_base_init()

for folder in FOLDERS:
    process_folder(folder)
