import re
import json

default = 0x3F76E4
table = {
    'Swamp': 0x617B64,
    'River': 0x3F76E4,
    'Ocean': 0x3F76E4,
    'Lukewarm Ocean': 0x45ADF2,
    'Warm Ocean': 0x43D5EE,
    'Cold Ocean': 0x3D57D6,
    'Frozen River': 0x3938C9,
    'Frozen Ocean': 0x3938C9,
}
spec = {
    'Swamp': (
        {'Fixed': 0x4C763C}, {'Fixed': 0x4C763C},
    ),
    'Dark Forest': (
        {'Average': 0x28340A}, {'Average': 0x28340A}
    ),
    'Badlands': (
        {'Fixed': 0x90814D}, {'Fixed': 0x9E814D},
    )
}

def parse_cpp(path):
    pattern = re.compile(r'\s*\{\s*/\*\s*(\d{1,3})\s*\*/\s*\"(.+)\",\s*([-\d\.]+)f,\s*([-\d\.]+)f,\s*(0x[\dABCDEF]{6}),\s*(0x[\dABCDEF]{6})\s*\},\s*')
    data = list()
    with open(path) as ifile:
        mark = 0
        line = ifile.readline()
        while line is not None:
            if mark == 0 and line.startswith('Biome gBiomes[256]={'):
                mark = 1
            if mark > 0 and line.startswith('};'):
                mark = -1
                break
            if mark > 0:
                m = pattern.match(line)
                if m is not None:
                    s = (
                        int(m.group(1)),
                        m.group(2),
                        float(m.group(3)),
                        float(m.group(4)),
                        int(m.group(5), 16),
                        int(m.group(6), 16),
                        table.get(m.group(2), default)
                    )
                    print(s)
                    data.append(s)
            line = ifile.readline()
    return data


def gen_source(path, data):
    with open(path, 'w') as ofile:
        ofile.write('// generate automatically\n')
        ofile.write('pub const BIOME_DATA: [(&\'static str, f32, f32, u32); 256] = [\n')
        ofile.write('    // (name, temperature, rainfall, water_color) \n')
        for s in data:
            ofile.write('    (/* %3d */ %-36s, %.2f, %.2f, 0x%X),\n' % (s[0], '"' + s[1] + '"', s[2], s[3], s[6]))
        ofile.write('];\n')
        ofile.write('\n')
        ofile.write('pub const COLORMAP_GRASS: &\'static [u8] = include_bytes!("grass.png");\n')
        ofile.write('\n')
        ofile.write('pub const COLORMAP_FOLIAGE: &\'static [u8] = include_bytes!("foliage.png");\n')  
        ofile.write('\n')

def gen_json(path, data):
    json_data = list()
    for s in data:
        obj = {
            'id': s[0],
            'name': s[1],
            'temperature': s[2],
            'rainfall': s[3],
            'watercolor': s[6],
        }
        if s[0] in (6,):
            ops = spec['Swamp']
            obj['ops_grass'] = ops[0]
            obj['ops_foliage'] = ops[1]
        elif s[0] in (29,):
            ops = spec['Dark Forest']
            obj['ops_grass'] = ops[0]
            obj['ops_foliage'] = ops[1]
        elif s[0] in (37, 38, 39, 165, 166, 167):
            ops = spec['Badlands']
            obj['ops_grass'] = ops[0]
            obj['ops_foliage'] = ops[1]
        json_data.append(obj)
    with open(path, 'w') as ofile:
        json.dump(json_data, ofile)

if __name__ == "__main__":
    data = parse_cpp('biomes.cpp')
    gen_json('biome.json', data)



