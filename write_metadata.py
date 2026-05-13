import math
import sys
import json
from PIL import Image
import numpy as np
fname = sys.argv[1]
radius = float(sys.argv[2])
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
    x_second_point = x_max
    y_second_point = height - 1
    while rgba[y_second_point, x_second_point, 3] < 128:
        y_second_point -= 1
    x_first_point = 0
    y_first_point = height - 1
    while rgba[y_first_point, x_first_point, 3] < 128:
        y_first_point -= 1
    
    l = ((x_second_point - x_first_point)**2 + (y_second_point - y_first_point)**2)**0.5 # distance between points
    h = (radius**2 - (l/2)**2)**0.5 # center perpendicular to line from first_point to second_point, circle center must be on it
    deltax = h / l * (y_second_point - y_first_point)
    deltay = - h / l * (x_second_point - x_first_point)
    x_center_l = (x_first_point + x_second_point) / 2
    y_center_l = (y_first_point + y_second_point) / 2
    x_center_circle = x_center_l + deltax
    y_center_circle = y_center_l + deltay

    theta1 = math.atan2((y_first_point - y_center_circle), (x_first_point - x_center_circle))
    theta2 = math.atan2((y_second_point - y_center_circle), (x_second_point - x_center_circle))
    

    d = {"fname": fname,
         "trunk_ratio": (trunk_x + 1) / width,
         "ymax_ratio": (y_max + 1) / width,
         "xmax_ratio": (x_max + 1) / width,
         "x_circle": x_center_circle / width,
         "y_circle": y_center_circle / width,
         "radius": radius / width,
         "theta_max": theta1 + 0.1,
         "theta_min": theta2 - 0.1}
    json_data = json.dumps(d)
    new_fname = fname[:-3] + "json"
    with open(new_fname, "w") as f:
        f.write(json_data)