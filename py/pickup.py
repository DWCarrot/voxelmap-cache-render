
import os
import shutil
import sys

FILELIST = [
    "index.json",
    "colormap.png",
    "weightmap.png",
    "grass.png",
    "foliage.png",
    "biome.json"
]


def current_dir(arg0):
    sp = os.path.split(os.path.abspath(arg0))
    return sp[0]

def main(args):
    tgtf = os.path.join(args[0], 'resource')
    try:
        os.mkdir(tgtf)
    except FileExistsError as e:
        pass
    for name in FILELIST:
        src = os.path.abspath(name)
        shutil.copy(src, tgtf)
    return 0

if __name__ == "__main__":
    cur = current_dir(sys.argv[0])
    os.chdir(cur)
    main(sys.argv[1:])