import sys
import json
from PIL import Image
import numpy as np
fname = sys.argv[1]
img = Image.open(fname).convert('RGBA')
width = img.width
height = img.height
if width != height:
    print("Error! Image must be square!")
else:
    rgba = np.array(img).astype('uint8')
    trunk_x = width - 1
    while rgba[0, trunk_x, 3] < 128:
        trunk_x -= 1
    y_max = height - 1
    while len((rgba[y_max, :, 3] > 128).nonzero()[0]) == 0:
        y_max -= 1
    x_max = width - 1
    while len((rgba[:, x_max, 3] > 128).nonzero()[0]) == 0:
        x_max -= 1
    d = {"fname": fname,
         "trunk_ratio": (trunk_x + 1) / width,
         "ymax_ratio": (y_max + 1) / width,
         "xmax_ratio": (x_max + 1) / width}
    json_data = json.dumps(d)
    new_fname = fname[:-3] + "json"
    with open(new_fname, "w") as f:
        f.write(json_data)