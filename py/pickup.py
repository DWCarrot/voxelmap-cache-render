
import os
import shutil
import sys

CUR = ''

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
        os.makedirs (tgtf)
    except FileExistsError as e:
        pass
    for name in FILELIST:
        src = os.path.abspath(os.path.join(CUR, name))
        shutil.copy(src, tgtf)
        print('copyed', src, 'to', tgtf)
    return 0

if __name__ == "__main__":
    CUR = current_dir(sys.argv[0])
    main(sys.argv[1:])