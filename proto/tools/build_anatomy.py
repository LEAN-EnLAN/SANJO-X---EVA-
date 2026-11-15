# Conversor propio: toma la geometría BodyParts3D (CC BY 4.0) empaquetada por Human Atlas
# y genera paquetes livianos (posiciones cuantizadas u16 por pieza + índices u16).
import json, struct, sys, os
src, out = sys.argv[1], sys.argv[2]
atlas = json.load(open(os.path.join(src, 'atlas.json')))
chunks = {}
def chunk(i):
    if i not in chunks: chunks[i] = open(os.path.join(src, f'body-{i}.bin'), 'rb').read()
    return chunks[i]
packs = {'core': {'integumentary','cardiac','respiratory','digestive','urinary','endocrine','sensory'},
         'frame': {'skeletal','nervous'}}
manifest = {'source': 'BodyParts3D 4.0 (DBCLS), CC BY 4.0, via Human Atlas packaging', 'packs': {}}
for pack, systems in packs.items():
    buf = bytearray(); parts = []
    for p in atlas['parts']:
        if p['system'] not in systems: continue
        b = chunk(p['chunk']); n = p['vertexCount']; m = p['indexCount']
        pos = struct.unpack_from(f'<{n*3}f', b, p['positions'])
        idx = struct.unpack_from(f'<{m}I', b, p['indices'])
        lo = [min(pos[a::3]) for a in range(3)]; hi = [max(pos[a::3]) for a in range(3)]
        sc = [(hi[a]-lo[a]) or 1e-9 for a in range(3)]
        q = [round((pos[i]-lo[i%3])/sc[i%3]*65535) for i in range(n*3)]
        off = len(buf)
        buf += struct.pack(f'<{n*3}H', *q)
        if len(buf) % 4: buf += b'\0' * (4 - len(buf) % 4)
        ioff = len(buf)
        buf += struct.pack(f'<{m}H', *idx)
        if len(buf) % 4: buf += b'\0' * (4 - len(buf) % 4)
        parts.append([p['id'], p['name'], p['system'], n, m, off, ioff, [round(v,5) for v in lo], [round(v,6) for v in sc]])
    open(os.path.join(out, f'anatomy-{pack}.bin'), 'wb').write(buf)
    manifest['packs'][pack] = {'file': f'data/anatomy-{pack}.bin', 'bytes': len(buf), 'parts': parts}
    print(pack, len(parts), 'partes', round(len(buf)/1e6, 2), 'MB')
json.dump(manifest, open(os.path.join(out, 'anatomy.json'), 'w'), separators=(',',':'))
print('manifest KB', os.path.getsize(os.path.join(out, 'anatomy.json'))//1024)
