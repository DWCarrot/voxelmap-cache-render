import sys
import copy
import re
import json
import zipfile

import numpy as np
import cv2



def empty_2d_array(x, y):
    return [ [None for j in range(y)] for i in range(x) ]

def _to_tuple(data):
    for k in data.keys():
        outer = data[k]
        for i in range(len(outer)):
            inner = outer[i]
            outer[i] = tuple(inner)
        data[k] = tuple(outer)
    return data

def _get_inv(table):
    inv = {
        'west':empty_2d_array(4, 4),
        'down':empty_2d_array(4, 4),
        'north':empty_2d_array(4, 4),
        'south':empty_2d_array(4, 4), 
        'up':empty_2d_array(4, 4),
        'east':empty_2d_array(4, 4)
    }
    for x in range(4):
        for y in range(4):
            for (k, l2d) in table.items():
                t = l2d[x][y]
                inv[t][x][y] = k
    return inv

ROTATE = _to_tuple({
    'west': [
        ['west', 'north', 'east', 'south'], # west
        ['west', 'north', 'east', 'south'], # west
        ['west', 'north', 'east', 'south'], # west
        ['west', 'north', 'east', 'south'],  # west
    ],
    'down': [
        ['down', 'down', 'down', 'down'], # down
        ['north', 'east', 'south', 'west'], # north
        ['up', 'up', 'up', 'up'], # up
        ['south', 'west', 'north', 'east'],  # south
    ],
    'north': [
        ['north', 'east', 'south', 'west'], # north
        ['up', 'up', 'up', 'up'], # up
        ['south', 'west', 'north', 'east'],  # south
        ['down', 'down', 'down', 'down'], # down
    ],
    'south': [
        ['south', 'west', 'north', 'east'],  # south
        ['down', 'down', 'down', 'down'], # down
        ['north', 'east', 'south', 'west'], # north
        ['up', 'up', 'up', 'up'], # up
    ],
    'up': [
        ['up', 'up', 'up', 'up'], # up
        ['south', 'west', 'north', 'east'],  # south
        ['down', 'down', 'down', 'down'], # down
        ['north', 'east', 'south', 'west'], # north
    ],
    'east': [
        ['east', 'south', 'west', 'north' ], # east
        ['east', 'south', 'west', 'north' ], # east
        ['east', 'south', 'west', 'north' ], # east
        ['east', 'south', 'west', 'north' ], # east
    ]
})
INV_ROTATE = _get_inv(ROTATE)
FACE_INDEX = {
    'west':(0,4,2,7), 
    'down':(0,1,4,5), 
    'north':(1,0,3,2),
    'south':(4,5,6,7), 
    'up':(6,7,2,3), 
    'east':(5,1,7,3)
}
FACE_AXIS = {
    'west': np.array([-1, 0, 0], np.float),
    'down': np.array([0, -1, 0], np.float),
    'north': np.array([0, 0, -1], np.float),
    'south': np.array([0, 0, 1], np.float),
    'up': np.array([0, 1, 0], np.float),
    'east': np.array([1, 0, 0], np.float),
}
FACE_ROTATE_FACTOR_X = {
    'west': 90,
    'down': 0,
    'north': 0,
    'south': 0,
    'up': 0,
    'east': -90,
}
FACE_ROTATE_FACTOR_Y = {
    'west': 0,
    'down': -90,
    'north': 0,
    'south': 0,
    'up': 90,
    'east': 0,
}

class AssetsLoader:

    def __init__(self, resources):
        if isinstance(resources, list):
            self.zips = [zipfile.ZipFile(resource, 'r') for resource in resources]
        else:
            self.zips = [zipfile.ZipFile(resources, 'r')]
        self.names = dict()
        for (i, rsc) in enumerate(self.zips):
            for name in rsc.namelist():
                self.names[name] = i

    def get_blocks(self):
        pattern = re.compile(r'assets/(\S+)/blockstates/(\S+).json')
        st = list()
        for name in self.names.keys():
            m = pattern.match(name)
            if m is not None:
                st.append((m.group(1), m.group(2)))
        st.sort()
        return st

    def get_blockstate(self, namespace, name):
        full = 'assets/%s/blockstates/%s.json' % (namespace, name)
        i = self.names.get(full)
        if i is not None:
            with self.zips[i].open(full) as ifile:
                return json.load(ifile)
        return None

    def get_model(self, namespace, name):
        full = 'assets/%s/models/%s.json' % (namespace, name)
        i = self.names.get(full)
        if i is not None:
            with self.zips[i].open(full) as ifile:
                return json.load(ifile)
        return None

    def get_texture(self, namespace, name):
        info = 'assets/%s/textures/%s.png.mcmeta' % (namespace, name)
        full = 'assets/%s/textures/%s.png' % (namespace, name)
        i = self.names.get(full)
        if i is not None:
            with self.zips[i].open(full) as ifile:
                buf = ifile.read()
                buf = np.frombuffer(buf, np.uint8)
                img = cv2.imdecode(buf, cv2.IMREAD_UNCHANGED)
                if info in self.names:
                    w = img.shape[1]
                    img = img[0:w,0:w,:]
                return img
        return None


class AppliedModel:

    def __init__(self, json_data):
        if isinstance(json_data, list):
            maxw = 0
            for one in json_data:
               w = one.get('weight', 1)
               if maxw < w:
                   maxw = w
                   self._parse_model(one)
        else:
            self._parse_model(json_data) 
        
    def _parse_model(self, one):
        self.model = one.get('model')
        self.x = int(one.get('x', 0))
        self.y = int(one.get('y', 0))
        self.uvlock = bool(one.get('uvlock', False))

    def get_faces(self, face):
        if isinstance(self.model, Model):
            x = self.x // 90
            y = self.y // 90
            original_face = INV_ROTATE[face][x][y]
            rotate = np.array([
                FACE_AXIS[ROTATE['east'][x][y]],
                FACE_AXIS[ROTATE['up'][x][y]],
                FACE_AXIS[ROTATE['south'][x][y]]
            ], np.float).T
            center = np.array([8, 8, 8], np.float)
            faces_tuple = self.model.get_faces(original_face)
            for (vs, fs) in faces_tuple:
                for v in vs:
                    v[:] = np.dot(rotate, v - center) + center
                cullface = fs.cullface
                if cullface is not None:
                    fs.cullface = ROTATE[cullface][x][y]
                if self.uvlock:
                    fs.rotation = (fs.rotation + 720 - x * FACE_ROTATE_FACTOR_X[face] - y * FACE_ROTATE_FACTOR_Y[face]) % 360
            return faces_tuple
        return None


class ModelProvider:

    def __init__(self, loader: AssetsLoader, namespace):
        self.namespace = namespace
        self.loader = loader
        self.cache = dict()

    # def set_namespace(self, namespace):
    #     self.namespace = namespace

    def get(self, name, nocache=False):
        namespace = self.namespace
        i = name.find(':')
        if i >= 0:
            namespace = name[:i]
            name = name[i+1:]
        if nocache:
            mdl = self.loader.get_model(namespace, name)
            if mdl is not None:
                mdl = Model(mdl)
        else:
            mdl = self.cache.get(namespace + ':' + name)
            if mdl is None:
                mdl = self.loader.get_model(namespace, name)
                if mdl is not None:
                    mdl = Model(mdl)
                    self.cache[namespace + ':' + name] = mdl
        return mdl


class TextureProvider:

    def __init__(self, loader: AssetsLoader, namespace):
        self.namespace = namespace
        self.loader = loader
        self.cache = dict()

    # def set_namespace(self, namespace):
    #     self.namespace = namespace

    def get(self, name, nocache=False):
        namespace = self.namespace
        i = name.find(':')
        if i >= 0:
            namespace = name[:i]
            name = name[i+1:]
        if nocache:
            tex = self.loader.get_texture(namespace, name)
            if tex is not None:
                tex = Model(tex)
        else:
            tex = self.cache.get(namespace + ':' + name)
            if tex is None:
                tex = self.loader.get_texture(namespace, name)
                if tex is not None:
                    self.cache[namespace + ':' + name] = tex
        return tex


class IdGen:

    def __init__(self):
        self.cache = [None] 

    def generate(self, applied_model: AppliedModel):
        self.cache.append(applied_model)
        return len(self.cache) - 1

    def total(self):
        return len(self.cache)

    def link(self, mdlpvd: ModelProvider):
        targets = dict()
        for am in self.cache[1:]:
            name = am.model
            mdl = targets.get(name)
            if mdl is None:
                mdl = mdlpvd.get(name, True)
                mdl.link(mdlpvd)
                print('load model:', name)
                targets[name] = mdl
            am.model = mdl
        return self.cache


class Blockstate:

    def __init__(self, json_data):
        self.props = dict()
        self.pairs = list()
        self.ids = list()
        if 'variants' in json_data:
            self._parse_variants(json_data['variants'])
            return
        if 'multipart' in json_data:
            self._parse_multipart(json_data['multipart'])
            return

    def serialize(self, id_gen: IdGen):
        self.ids = [0] * len(self.pairs)
        json_data = dict()
        if len(self.props) == 0:
            tp = 'single'
        elif isinstance(self.pairs[0][0], list):
            tp = 'multipart'
        else:
            tp = 'variants'
        if tp == 'single':
            value = id_gen.generate(self.pairs[0][1])
            self.ids[0] = value
            json_data[tp] = value
        else:
            keys = dict()
            for (k1, v1) in self.props.items():
                for (k2, v) in v1.items():
                    k = '%s=%s' % (k1, k2)
                    keys[k] = v
            if tp == 'variants':
                values = dict()
                for (i, pair) in enumerate(self.pairs):
                    name = str(pair[0])
                    value = id_gen.generate(pair[1])
                    self.ids[i] = value
                    values[name] = value                   
            else:
                values = list()
                for (i, pair) in enumerate(self.pairs):
                    value = id_gen.generate(pair[1])
                    self.ids[i] = value
                    value = {'when': pair[0], 'apply': value}
                    values.append(value)
            json_data[tp] = {'keys': keys, 'values': values}
        return json_data


    def _parse_variants(self, json_data):
        for k in json_data.keys():
            if k == '':
                v = json_data['']
                self.pairs.append((0x00, AppliedModel(v)))
                return
            for p in k.split(','):
                tmp = p.split('=')
                pname = tmp[0]
                pvalue = tmp[1]
                key = self.props.get(pname)
                if key is None:
                    key = dict()
                    self.props[pname] = key
                key[pvalue] = 0x00
        self._gen_key()
        for (k, v) in json_data.items():
            msk = 0x00
            for p in k.split(','):
                tmp = p.split('=')
                pname = tmp[0]
                pvalue = tmp[1]
                m = self.props[pname][pvalue]
                msk |= m
            self.pairs.append((msk, AppliedModel(v))) 

    def _parse_multipart(self, json_data):
        for item in json_data:
            cond = item.get('when')
            if cond is not None:
                or_cond = cond.get('OR')
                if or_cond is not None:
                    for c in or_cond:
                        self._add_key_one(c)
                else:
                    self._add_key_one(cond)
        self._gen_key()
        for item in json_data:
            cond = item.get('when')
            keys = list()
            if cond is not None:
                or_cond = cond.get('OR')
                if or_cond is not None:
                    for c in or_cond:
                        keys.extend(self._cal_key(c))
                else:
                    keys.extend(self._cal_key(cond))
            if len(keys) == 0:
                keys.append(0)
            self.pairs.append((keys, AppliedModel(item['apply'])))
        
    def _add_key_one(self, cond):
        for (k, v) in cond.items():
            v = str(v)
            key = self.props.get(k)
            if key is None:
                key = dict()
                self.props[k] = key
            for value in v.split('|'):
                key[value] = 0x00

    def _gen_key(self):
        msk = 0x01
        for (k, v) in self.props.items():
            for p in v.keys():
                v[p] = msk
                msk <<= 1

    def _cal_key(self, cond):
        msks = [0x00]
        for (k, v) in cond.items():
            v = str(v)
            propname = self.props[k]
            if '|' in v:
                new_msks = []
                for value in v.split('|'):
                    key = propname[value]
                    tmp = [i | key for i in msks]
                    new_msks.extend(tmp)
                msks = new_msks
            else:
                count = len(msks)
                key = propname[v]
                for i in range(count):
                    msks[i] |= key
        return msks


 

class Face:

    def __init__(self, json_data):
        self.uv = np.array(json_data.get('uv', [0, 0, 16, 16]), np.float)
        self.texture = json_data.get('texture')
        self.cullface = json_data.get('cullface', None)
        self.rotation = json_data.get('rotation', 0)
        self.tintindex = json_data.get('tintindex', None)

    def link(self, textures):
        while self.texture.startswith('#'):
            t = self.texture[1:]
            self.texture = textures.get(t, self.texture)


class Element:

    def __init__(self, json_data):
        self.v_from = np.array(json_data.get('from'), np.float)
        self.v_to = np.array(json_data.get('to'), np.float)
        self.rotation = None ###ignore
        self.shade = json_data.get('shade', False)
        self.faces = dict()
        faces = json_data.get('faces')
        if faces is not None:
            for (k, v) in faces.items():
                self.faces[k] = Face(v)

    def link(self, textures):
        for (k, v) in self.faces.items():
            v.link(textures)
 
    def get_vertex(self, index: int):
        v = np.copy(self.v_from)
        if index & 0x1 > 0:
            v[0] = self.v_to[0]
        if index & 0x2 > 0:
            v[1] = self.v_to[1]
        if index & 0x4 > 0:
            v[2] = self.v_to[2]
        return v

    def get_face(self, face):
        return self.faces.get(face)

    def get_face_vertex(self, face):
        return tuple([self.get_vertex(i) for i in FACE_INDEX[face]])

    def get_face_area(self, vertexs):
        diff = vertexs[2] - vertexs[0]
        return np.prod(diff[diff > 0])

class Model:

    def __init__(self, json_data):
        self.parent = json_data.get('parent', None)
        self.ambientocclusion = json_data.get('ambientocclusion', None)
        self.display = None ###ignore
        textures = json_data.get('textures')
        if textures is not None:
            self.textures = textures
        else:
            self.textures = dict()
        elements = json_data.get('elements')
        if elements is not None:
            self.elements = [Element(e) for e in elements]
        else:
            self.elements = None

    def link(self, model_cache):
        if self.parent is not None:
            stk = [self]
            parent = self.parent
            while parent is not None:
                mdl = model_cache.get(parent)
                stk.append(copy.deepcopy(mdl))
                parent = mdl.parent
            parent = stk.pop()
            while len(stk) > 0:
                this = stk.pop()
                if this.ambientocclusion is None:
                    this.ambientocclusion = parent.ambientocclusion
                for (k, v) in this.textures.items():
                    parent.textures[k] = v
                this.textures = parent.textures
                if this.elements is None:
                    this.elements = parent.elements
                parent = this
        del self.parent
        if self.ambientocclusion is None:
            self.ambientocclusion = True
        if self.elements is None:
            self.elements = list()
        for e in self.elements:
            e.link(self.textures)

    def get_faces(self, face):
        res = list()
        for e in self.elements:
            facetex = e.get_face(face)
            if facetex is not None:
                vertex = e.get_face_vertex(face)
                res.append((vertex, copy.copy(facetex)))
        return res


###
###
###


EQUIV_ROTATE1 = [0, 270, 90, 180]
EQUIV_ROTATE2 = [90, 180, 0, 270]


class FullRenderer:

    def __init__(self, count, linewidth, tex_pvd):
        self.iw = linewidth
        self.ih = (count + linewidth - 1) // linewidth
        self.tex_pvd = tex_pvd
        self.img = np.zeros((self.ih * 16, self.iw * 16, 4), np.uint8)
        self.heightmap = np.zeros((self.ih, self.iw), np.uint8) #  height x 8

    def draw(self, fs_tuples, index):
        if index >= self.ih * self.iw:
            return False
        ix = (index % self.iw)
        iy = (index // self.iw)
        offset = np.array((ix, iy, ix, iy), np.int32) * 16
        elements = [FullRenderer.rectify(vertexs, face) for (vertexs, face) in fs_tuples]
        sorted(elements, key=lambda x: x[4])
        height = 0
        if len(elements) == 0:
            return False
        for (src_uv, tex_uv, tex, r, h) in elements:
            tex = self.tex_pvd.get(tex)
            if tex is None:
                continue
            tex_uv = tex_uv / 16.0 * np.array((tex.shape[1], tex.shape[0], tex.shape[1], tex.shape[0]), np.float)
            tex_uv[0:2] = np.floor(tex_uv[0:2])
            tex_uv[2:4] = np.ceil(tex_uv[2:4])
            tex_uv = np.array(tex_uv, np.int32)
            tex = tex[tex_uv[1]:tex_uv[3],tex_uv[0]:tex_uv[2],:]
            r = -(r // 90)
            tex = np.rot90(tex, r)
            src_uv[0:2] = np.floor(src_uv[0:2])
            src_uv[2:4] = np.ceil(src_uv[2:4])
            src_uv = np.array(src_uv, np.int32)
            tex = cv2.resize(tex, (src_uv[2] - src_uv[0], src_uv[3] - src_uv[1]), interpolation=cv2.INTER_NEAREST)
            src_uv = src_uv + offset
            if tex.shape[2] < self.img.shape[2]:
                self.img[src_uv[1]:src_uv[3],src_uv[0]:src_uv[2],0:3] = tex
                self.img[src_uv[1]:src_uv[3],src_uv[0]:src_uv[2],3] = 255
            else:
                bg = np.array(self.img[src_uv[1]:src_uv[3],src_uv[0]:src_uv[2], :], np.float) / 255
                fg = np.array(tex, np.float) / 255
                FullRenderer.blend(bg, fg)
                self.img[src_uv[1]:src_uv[3],src_uv[0]:src_uv[2],:] = np.array(bg * 255, np.uint8)
            height = h
        self.heightmap[iy, ix] = int(height * 8)
        return True

    @staticmethod
    def blend(bg: np.ndarray, fg: np.ndarray):
        fg_a = fg[:,:,3]
        factor = fg_a
        fg[:,:,0] *= factor
        fg[:,:,1] *= factor
        fg[:,:,2] *= factor
        bg_a = bg[:,:,3]
        factor = bg_a * (1.0 - fg_a)
        bg[:,:,0] *= factor
        bg[:,:,1] *= factor
        bg[:,:,2] *= factor
        factor = fg_a + bg_a * (1.0 - fg_a)
        mask = factor > 0
        t = fg[:,:,0] + bg[:,:,0]
        t[mask] /= factor[mask]
        t[~mask] = 0.0
        bg[:,:,0] = t
        t = fg[:,:,1] + bg[:,:,1]
        t[mask] /= factor[mask]
        t[~mask] = 0.0
        bg[:,:,1] = t
        t = fg[:,:,2] + bg[:,:,2]
        t[mask] /= factor[mask]
        t[~mask] = 0.0
        bg[:,:,2] = t
        bg[:,:,3] = factor
        

    @staticmethod
    def rectify(vertexs, face):
        h = vertexs[0][1]
        (src_uv ,src_r) = FullRenderer.rectify_verts(vertexs)
        (tex_uv, tex_r) = FullRenderer.rectify_uv(face.uv)
        r = (src_r + tex_r + face.rotation) % 360
        tex = face.texture
        return (src_uv, tex_uv, tex, r, h)

    @staticmethod
    def rectify_uv(uv):
        k = 0x0
        uv = np.copy(uv)
        if uv[0] > uv[2]:
            k |= 0x1
            t = uv[0]
            uv[0] = uv[2]
            uv[2] = t
        if uv[1] > uv[3]:
            k |= 0x2
            t = uv[1]
            uv[1] = uv[3]
            uv[3] = t
        return (uv, EQUIV_ROTATE1[k])

    @staticmethod
    def rectify_verts(vertexs):
        vmin = vertexs[0]
        imin = 0
        vmax = vertexs[0]
        imax = 0
        c = 1
        for v in vertexs[1:]:
            if v[0] <= vmin[0] and v[2] <= vmin[2]:
                imin = c
                vmin = v
            if v[0] >= vmax[0] and v[2] >= vmax[2]:
                imax = c
                vmax = v
            c += 1
        return (np.array((vmin[0], vmin[2], vmax[0], vmax[2])), EQUIV_ROTATE2[imin])

    @staticmethod
    def color_extraction_weight():
        return np.ones((16,16), np.float)

    def color_extraction(self):
        colormap = np.zeros((self.ih, self.iw, 4), np.uint8)
        weightmap = np.zeros((self.ih, self.iw), np.uint8)
        swi = FullRenderer.color_extraction_weight()
        for x in range(self.iw):
            ix = x * 16
            for y in range(self.ih):
                iy = y * 16
                color = np.zeros((4,), np.uint8)
                selected = self.img[iy:iy+16,ix:ix+16,:]
                mask = selected[:,:,3] > 0
                if np.any(mask):
                    for c in range(4):
                        channel = np.array(selected[:,:,c], np.float)
                        tmp = channel * swi
                        color[c] = np.sum(tmp[mask]) / np.sum(swi[mask])
                colormap[y, x, :] = color[:]
                weightmap[y, x] = max(np.count_nonzero(mask) - 1, 0)
        return (colormap, weightmap)
                    


###
###
###

import argparse
import logging
from os import path


if __name__ == "__main__":
    
    parser = argparse.ArgumentParser()
    parser.add_argument('assets', type=str, help='minecraft resourcepack (or version.jar)', nargs='+')
    parser.add_argument('-l', '--log', type=str, help='log output: file or STDOUT', default='STDOUT', required=False)  
    parser.add_argument('-w', '--linewidth', type=int, help='number of element every line', default=32, required=False)
    args = parser.parse_args()
    

    if args.log == 'STDOUT':
        logging.basicConfig(level=logging.INFO)
    else:
        logging.basicConfig(filename=args.log, level=logging.INFO)


    loader = AssetsLoader(args.assets)
    cache = dict()
    idgen = IdGen()
    logging.info('> generate index')
    with open('index.json', 'w') as ofile:
        whole = dict()
        data = dict()
        for (namespace, block) in loader.get_blocks():
            name = namespace + ':' + block
            bs = loader.get_blockstate(namespace, block)
            bs = Blockstate(bs)
            cache[name] = bs
            data[name] = bs.serialize(idgen)
            logging.info('load blockstate   {}'.format(name))
        whole['data'] = data
        whole['config'] = {
            'linewidth': args.linewidth
        }
        json.dump(whole, ofile)
        logging.info('> write index')
    
    count = idgen.total()
    del idgen

    logging.info('> generate model')
    model_cache = dict()
    mdlpvd = ModelProvider(loader, 'minecraft')
    for (name, bs) in cache.items():
        i = name.find(':')
        namespace = name[:i]
        name = name[i+1:]
        # mdlpvd.set_namespace(namespace)
        for (bspair, index) in zip(bs.pairs, bs.ids):
            applied_model = bspair[1]
            mdlname = namespace + ':' + applied_model.model
            mdl = model_cache.get(mdlname)
            if mdl is None:
                mdl = mdlpvd.get(applied_model.model, True)
                if mdl is None:
                    logging.warning("model is None: {}".format(applied_model.model))
                    continue
                mdl.link(mdlpvd)
                logging.info('load model  {}'.format(mdlname))
                model_cache[mdlname] = mdl
            applied_model.model = mdl

    del model_cache
    del mdlpvd

    logging.info('> render')
    tex_pvd = TextureProvider(loader, 'minecraft')
    renderer = FullRenderer(count, args.linewidth, tex_pvd)
    for (name, bs) in cache.items():
        i = name.find(':')
        namespace = name[:i]
        name = name[i+1:]
        # tex_pvd.set_namespace(namespace)
        for (bspair, index) in zip(bs.pairs, bs.ids):
            applied_model = bspair[1]
            ts = applied_model.get_faces('up')
            try:
                if renderer.draw(ts, index):
                    logging.info('draw [{}] {}:{}'.format(index, namespace, name))
                else:
                    logging.warning('empty [{}] {}:{}'.format(index, namespace, name))
            except Exception as e:
                logging.warning('[{}] {}:{}   {}'.format(index, namespace, name, e))

    cv2.imwrite('baked.png', renderer.img)
    cv2.imwrite('heightmap.png', renderer.heightmap)
    (cmap, wmap) = renderer.color_extraction()
    cv2.imwrite('colormap.png', cmap)
    cv2.imwrite('weightmap.png', wmap)
    c_grass = loader.get_texture('minecraft', 'colormap/grass')
    cv2.imwrite('grass.png', c_grass[:,:,0:3])
    c_foliage = loader.get_texture('minecraft', 'colormap/foliage')
    cv2.imwrite('foliage.png', c_foliage[:,:,0:3])
    pass

'''
if __name__ == "__main__":
    ip = input('> jar: ')
    loader = AssetsLoader(ip)
    idgen = IdGen()
    mdlpvd = ModelProvider(loader, 'minecraft')
    with open('index.json', 'w') as ofile:
        whole = dict()
        for (ns, b) in loader.get_blocks():
            #print('load', b)
            bs = loader.get_blockstate(ns, b)
            #print('parse', b)
            bs = Blockstate(bs)
            #print('write', b)
            data = bs.serialize(idgen)
            name = '%s:%s' % (ns, b)
            whole[name] = data
        json.dump(whole, ofile)
    ls = idgen.link(mdlpvd)

    \'''   
    with open('text.txt', 'w') as ofile:
        for s in ls[1792:1796]:
            for (vs, fs) in s.get_faces('up'):
                ofile.write(str(vs))
                ofile.write('\n')
                ofile.write('%s %s %d' % (str(fs.uv), fs.texture, fs.rotation))
                ofile.write('\n')
            ofile.write('\n')
            ofile.write('\n')
    \'''
    renderer = FullRenderer(len(ls), 32, TextureProvider(loader, 'minecraft'))
    ofile = open('run.log', 'w')
    c = 1 - 1
    for s in ls[1:]:
        c += 1
        ts = s.get_faces('up')
        try:
            renderer.draw(ts, c)
        except Exception as e:
            ofile.write('[%d] %s\n' % (c, e))
        
    ofile.close()
    cv2.imwrite('baked.png', renderer.img)
    cv2.imwrite('heightmap.png', renderer.heightmap)
    (cmap, wmap) = renderer.color_extraction()
    cv2.imwrite('colormap.png', cmap)
    cv2.imwrite('weightmap.png', wmap)
    pass
'''