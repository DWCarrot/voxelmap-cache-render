import cv2
import numpy as np
import os
import re
import argparse

def list_files(path):
    pattern = re.compile(r'(-?\d+),(-?\d+).png')
    res = list()
    rg = list()  #[xmin ymin xmax ymax]
    for (dirpath, dirnames, filenames) in os.walk(path):
        for filename in filenames:
            m = pattern.match(filename)
            if m is not None:
                x = int(m.group(1))
                y = int(m.group(2))
                p = os.path.join(dirpath, filename)
                res.append((x,y,p))
                if len(rg) == 0:
                    rg.append(x)
                    rg.append(y)
                    rg.append(x)
                    rg.append(y)
                else:
                    if rg[0] > x:
                        rg[0] = x
                    if rg[1] > y:
                        rg[1] = y
                    if rg[2] < x:
                        rg[2] = x
                    if rg[3] < y:
                        rg[3] = y
    rg = (rg[0], rg[1], rg[2] + 1, rg[3] + 1)
    return (res, rg)
                
                
def merge(res, rg):
    st = np.array((256, 256), dtype=np.int32)
    rg = np.array(rg, dtype=np.int32)
    sz = (rg[2:4] - rg[0:2]) * st
    img = np.zeros((sz[1], sz[0], 4), dtype=np.uint8)
    st = np.array((st[0], st[1], st[0], st[1]), dtype=np.int32)
    sz = np.array((rg[0], rg[1], rg[0], rg[1]), dtype=np.int32)
    for (x, z, path) in res:
        if x < rg[0] or z < rg[1] or x >= rg[2] or z >= rg[3]:
            continue
        tg = np.array((x, z, x + 1, z + 1), dtype=np.int32)
        tg = (tg - sz) * st
        part = cv2.imread(path, flags=cv2.IMREAD_UNCHANGED)
        if part is None:
            continue
        img[tg[1]:tg[3],tg[0]:tg[2],:] = part[:,:,:]
    return img



if __name__ == "__main__":

    parser = argparse.ArgumentParser()
    parser.add_argument('input_dir', type=str)
    parser.add_argument('-o', '--output_file', type=str)
    parser.add_argument('-r', '--range', type=str) # xmin,ymin;xmax,ymax

    args = parser.parse_args()

    (res, rg) = list_files(args.input_dir)
    if not (args.range == 'max'):
        sp = args.range.split(' ')
        p1 = sp[0:2]
        xmin = int(p1[0])
        ymin = int(p1[1])
        p2 = sp[2:4]
        xmax = int(p2[0]) + 1
        ymax = int(p2[1]) + 1
        rg = (xmin, ymin, xmax, ymax)

    h = merge(res, rg)
    cv2.imwrite(args.output_file, h)

    pass