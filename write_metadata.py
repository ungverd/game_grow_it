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
    x = width - 1
    while rgba[0, x, 3] < 128:
        x -= 1
    d = {"fname": fname,
         "trunk_ratio": (x + 1) / width}
    json_data = json.dumps(d)
    new_fname = fname[:-3] + "json"
    with open(new_fname, "w") as f:
        f.write(json_data)