import sys
from PIL import Image
import numpy as np
fname = sys.argv[1]
res_fname = f"{fname[:-4]}_square.png"
img = Image.open(fname).convert('RGBA')
rgb = np.array(img).astype('uint8')
h, w = rgb.shape[0], rgb.shape[1]
print(h, w)
if h > w:
    addition = np.zeros((h, h-w, 4), dtype='uint8')
    new_rgb = np.concatenate((rgb, addition), axis=1)
elif w > h:
    addition = np.zeros((w-h, w, 4), dtype='uint8')
    new_rgb = np.concatenate((rgb, addition), axis=0)
else:
    new_rgb = rgb
new_img = Image.fromarray(new_rgb.astype('uint8'), 'RGBA')
new_img.save(res_fname)