# voxelmap-cache-render
Offline renderer for Minecraft voxelmap cache, with python (generate colormap) and Rust (fast render)



## rust renderer
### file struct
```bash
./
   [executable-binary-file]
   resource/
       biome.json
       foliage.png
       grass.png
       index.json
       colormap.png
       weightmap.png
```
these files can be found in `py/`
### usage
1. generate picture from `.minecraft[/versions/<version>]/mods/.mods/mamiyaotaru/voxelmap/cache/<server>/<world>/overworld/`

```bash
USAGE:
    voxelmapcache.exe render --input_dir <input_dir> --output_dir <output_dir> [OPTIONS]

    -i, --input_dir <input_dir>      input folder
    -o, --output_dir <output_dir>    output folder
OPTIONS:
    --env_lit <env_light>        environment light, from 0 to 15, default is 15
    --gamma <gamma>              gamma for gamma correction, default is 1.0
    -t, --thread <thread>        use multi-thread and set thread number, default is 1
```

2. generate map tiles with pictures from `step 1`
```bash
USAGE
    voxelmapcache.exe tile --input_dir <input_dir> --output_dir <output_dir> --path_mode <path_mode> [OPTIONS]

    -i, --input_dir <input_dir>      input folder
    -o, --output_dir <output_dir>    output folder
    --path_mode <path_mode>      generated path mode, can be "layer:<start>,<step>,<stop>"
        example: layer mode, the original scale is marked as 5 and the max-level scale is marked as 0
            => "layer:5,-1,0"
        example: layer mode, the original scale is marked as 0 and the max-level scale is marked as 3
            => "layer:0,1,3"
OPTIONS:
    --filter <filter>                filter used in scale, can be "nearest", "triangle", "gaussian", "catmullrom", "lanczos3"; default is "nearest"
    --use_multi_thread               whether to use multi-thread; if set, use fixed 4 threads
```

## python colormap generator

1. biomes_gen
2. cache_gen