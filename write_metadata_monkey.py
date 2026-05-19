import json
fname = "monkey_anim_L.png"

width = 1024
height = 1024
frame_width = 105
frame_height = 73
    

d = {"fname": fname,
     "width": width,
     "height": height,
     "frame_width": frame_width,
     "frame_height": frame_height,
     "n_frames": 120,
     "time_loop": 0.5,
     "advance_loop": frame_width * 1.4,
     }
json_data = json.dumps(d)
new_fname = fname[:-3] + "json"
with open(new_fname, "w") as f:
    f.write(json_data)