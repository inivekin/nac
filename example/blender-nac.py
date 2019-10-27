import sys
sys.path.insert(1, '/home/inivekin/code/nac/build/lib/')

import bpy

from example._native import ffi, lib

def get_grease_pencil(gpencil_obj_name='GPencil') -> bpy.types.GreasePencil:
    """
    Return the grease-pencil object with the given name. Initialize one if not already present.
    :param gpencil_obj_name: name/key of the grease pencil object in the scene
    """

    # If not present already, create grease pencil object
    if gpencil_obj_name not in bpy.context.scene.objects:
        # bpy.ops.object.gpencil_add(view_align=False, location=(0, 0, 0), type='EMPTY')
        bpy.ops.object.gpencil_add(location=(0, 0, 0), type='EMPTY')
        # rename grease pencil
        bpy.context.scene.objects[-1].name = gpencil_obj_name

    # Get grease pencil object
    gpencil = bpy.context.scene.objects[gpencil_obj_name]

    return gpencil


def get_grease_pencil_layer(gpencil: bpy.types.GreasePencil, gpencil_layer_name='GP_Layer',
                            clear_layer=False) -> bpy.types.GPencilLayer:
    """
    Return the grease-pencil layer with the given name. Create one if not already present.
    :param gpencil: grease-pencil object for the layer data
    :param gpencil_layer_name: name/key of the grease pencil layer
    :param clear_layer: whether to clear all previous layer data
    """

    # Get grease pencil layer or create one if none exists
    if gpencil.data.layers and gpencil_layer_name in gpencil.data.layers:
        gpencil_layer = gpencil.data.layers[gpencil_layer_name]
    else:
        gpencil_layer = gpencil.data.layers.new(gpencil_layer_name, set_active=True)

    if clear_layer:
        gpencil_layer.clear()  # clear all previous layer data

    # bpy.ops.gpencil.paintmode_toggle()  # need to trigger otherwise there is no frame

    return gpencil_layer


# Util for default behavior merging previous two methods
def init_grease_pencil(gpencil_obj_name='GPencil', gpencil_layer_name='GP_Layer',
                       clear_layer=True) -> bpy.types.GPencilLayer:
    gpencil = get_grease_pencil(gpencil_obj_name)
    gpencil_layer = get_grease_pencil_layer(gpencil, gpencil_layer_name, clear_layer=clear_layer)
    return gpencil_layer

def draw_line(gp_frame, p0: tuple, p1: tuple):
    # Init new stroke
    gp_stroke = gp_frame.strokes.new()
    gp_stroke.display_mode = '3DSPACE'  # allows for editing

    # Define stroke geometry
    gp_stroke.points.add(count=2)
    gp_stroke.points[0].co = p0
    gp_stroke.points[1].co = p1
    return gp_stroke


NUM_FRAMES = 120
FRAMES_SPACING = 1  # distance between frames
bpy.context.scene.frame_start = 0
bpy.context.scene.frame_end = NUM_FRAMES*FRAMES_SPACING

#cloth_ptr = lib.get_cloth_mesh(5,10,10)
cloth_ptr = lib.get_woven_cloth_mesh(5,20,5)
gp_layer = init_grease_pencil()

for frame in range(NUM_FRAMES):
    gp_frame = gp_layer.frames.new(frame*FRAMES_SPACING)
    node_count = lib.mesh_node_count(cloth_ptr)
    node_pos_d = None
    node_positions = [lib.mesh_node_position(cloth_ptr, i) for i in range(node_count)]
    
    #node_pos_d = None
    #for node_pos in node_positions:
    #    if node_pos_d:
    #        draw_line(gp_frame, (0, node_pos.x, node_pos.y), (0, node_pos_d.x, node_pos_d.y))
    #    node_pos_d = node_pos


    inter_con_count = lib.get_cloth_interconnector_count(cloth_ptr)
    constraint_counts = [lib.get_interconnector_constraint_count(cloth_ptr, index) for index in range(inter_con_count)]
        
    connector_positions = []
    for inter_index in range(inter_con_count):
        connector_positions += [lib.get_interconnector_constraint(cloth_ptr, inter_index, index) for index in range(constraint_counts[inter_index])]
    
    for node in connector_positions:
        draw_line(gp_frame, (0, node.x, node.y), (0, node.dx, node.dy))
    
    lib.update_cloth_mesh(cloth_ptr, 0.0016, 3)

lib.mesh_free(cloth_ptr)