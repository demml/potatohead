# Copies the _scouter.pyi stub file to a well-defined name so that mkdocstrings can find it.import os
import shutil
import os

BASE_DIR = os.path.join(os.path.dirname(__file__), "../python/potato_head")


# copy the _scouter.pyi to stubs.pyi
def create_stub():
    src_path = os.path.join(BASE_DIR, "_potato_head.pyi")
    dest_path = os.path.join(BASE_DIR, "stubs.pyi")
    shutil.copyfile(src_path, dest_path)
    print(f"Copied {src_path} to {dest_path}")


if __name__ == "__main__":
    create_stub()
