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
### usage
1. generate picture from `.minecraft[/versions/<version>]/mods/.mods/mamiyaotaru/voxelmap/cache/<server>/<world>/overworld/`



## python colormap generator

1. biomes_gen
2. cache_gen