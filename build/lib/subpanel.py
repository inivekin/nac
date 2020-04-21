import bpy
from bpy.types import (
    Panel,
)

import sys
sys.path.insert(1, '/home/inivekin/code/deprecated-nac/nac/build/lib/')

import math

from example._native import ffi, lib


GROUND_BOUND=-0.6


def get_grease_pencil(gpencil_obj_name='GPencil') -> bpy.types.GreasePencil:
    """
    Return the grease-pencil object with the given name. Initialize one if not already present.
    :param gpencil_obj_name: name/key of the grease pencil object in the scene
    """

    # If not present already, create grease pencil object
    if gpencil_obj_name not in bpy.context.scene.objects:
        # bpy.ops.object.gpencil_add(view_align=False, location=(0, 0, 0), type='EMPTY')
        bpy.ops.object.gpencil_add()
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

def draw_stroke(gp_frame, points, cyclic = False, material_index = 0):
    gp_stroke = gp_frame.strokes.new()
    gp_stroke.display_mode = '3DSPACE'
    gp_stroke.draw_cyclic = cyclic
    gp_stroke.material_index = material_index
    
    gp_stroke.points.add(count=len(points))
    for empty_point, new_point in zip(gp_stroke.points, points):
        empty_point.co = new_point

class NAC_panel:
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "data"
    
class NAC_MeshList(bpy.types.UIList):
    def draw_item(self, context, layout, data, item, icon, active_data, active_propname):
        ob = data
        slot = item
        ma = slot.material
        # draw_item must handle the three layout types... Usually 'DEFAULT' and 'COMPACT' can share the same code.
        if self.layout_type in {'DEFAULT', 'COMPACT'}:
            # You should always start your row layout by a label (icon + text), or a non-embossed text field,
            # this will also make the row easily selectable in the list! The later also enables ctrl-click rename.
            # We use icon_value of label, as our given icon is an integer value, not an enum ID.
            # Note "data" names should never be translated!
            if ma:
                layout.prop(ma, "name", text="", emboss=False, icon_value=icon)
            else:
                layout.label(text="", translate=False, icon_value=icon)
            # And now we can add other UI stuff...
            # Here, we add nodes info if this material uses (old!) shading nodes.
            if ma and not context.scene.render.use_shading_nodes:
                manode = ma.active_node_material
                if manode:
                    # The static method UILayout.icon returns the integer value of the icon ID "computed" for the given
                    # RNA object.
                    layout.label(text="Node %s" % manode.name, translate=False, icon_value=layout.icon(manode))
                elif ma.use_nodes:
                    layout.label(text="Node <none>", translate=False)
                else:
                    layout.label(text="")
        # 'GRID' layout type should be as compact as possible (typically a single icon!).
        elif self.layout_type in {'GRID'}:
            layout.alignment = 'CENTER'
            layout.label(text="", icon_value=icon)


class NAC_constraints(NAC_panel, Panel):
    bl_idname = "gp_cloth_panel"
    bl_label = "GPencil Cloth Simulator"
    COMPAT_ENGINES = {'BLENDER_RENDER', 'BLENDER_EEVEE', 'BLENDER_WORKBENCH'}

    #@classmethod
    #def poll(cls, context):
    #    ob = context.object
    #    return (ob and ob.rigid_body_constraint and context.engine in cls.COMPAT_ENGINES)

    def draw(self, context):
        #self.layout.use_property_split = True
        obj = context.object        

        row = self.layout.row()
        layer_rows = 7
        
        col = row.column()
        #col.template_list("NAC_MeshList", "", obj, "material_slots", obj, "active_material_index", rows=layer_rows)
        #col.template_list("GPENCIL_UL_layer", "", gpd, "layers", gpd.layers, "active_index",
        #                  rows=layer_rows, sort_reverse=True, sort_lock=True)

        col = row.column()
        sub = col.column(align=True)
        sub.operator("gpencil.stringify", icon='ADD', text="")
        #sub.operator("gpencil.add_gp_cloth_mesh", icon='REMOVE', text="")

class addMesh(bpy.types.Operator):
    bl_idname = "gpencil.add_gp_cloth_mesh"
    bl_label = "gp_cloth_mesh"
    
    def execute(self, context):
        NUM_FRAMES = 400
        FRAMES_SPACING=1
        gp_layer = init_grease_pencil()
        cloth_ptr = lib.get_cloth_mesh_field(5,10,5,98,-0.5,0.5)
        cloth_sims = bpy.data.meshes.new("cloth")
        cloth_sims.pointer = cloth_ptr
        
        
        for frame in range(NUM_FRAMES):
            gp_frame = gp_layer.frames.new(frame*FRAMES_SPACING)
            node_count = lib.mesh_node_count(cloth_ptr)
            node_positions = [lib.mesh_node_position(cloth_ptr, i) for i in range(node_count)]

            inter_con_count = lib.get_cloth_interconnector_count(cloth_ptr)
            constraint_counts = [lib.get_interconnector_constraint_count(cloth_ptr, index) for index in range(inter_con_count)]
            
            if frame == 100:
                name = ffi.new('char[]', bytes("wind", 'utf-8'))
                lib.remove_constraint_by_id(cloth_ptr, name)
                
            connector_positions = []
            for inter_index in range(inter_con_count):
                connector_positions += [lib.get_interconnector_constraint(cloth_ptr, inter_index, index) for index in range(constraint_counts[inter_index])]
            
            #for node in connector_positions:
            #    draw_line(gp_frame, (node.x, 0, node.y), (node.dx, 0, node.dy))
            node = [(node.x, 0, node.y) for node in connector_positions]
            draw_stroke(gp_frame, nodes)
            
            lib.update_cloth_mesh(cloth_ptr, 0.005, 5)

        #lib.mesh_free(cloth_ptr)

        return {'FINISHED'}


class Stringify(bpy.types.Operator):
    bl_idname = "gpencil.stringify"
    bl_label = "strinigifier"
    
    def execute(self, context):
        bpy.ops.object.mode_set(mode='OBJECT')
        mesh1 = lib.empty_cloth()
        print("Clothifying lowerleft")
        nodes1 = clothify_stroke(mesh1, 'lowerleft')
        print("Clothifying lowerright")
        mesh2 = lib.empty_cloth()
        nodes2 = clothify_stroke(mesh2, 'lowerright')
        print("Clothifying upperleft")
        mesh3 = lib.empty_cloth()
        nodes3 = clothify_stroke(mesh3, 'upperleft')
        print("Clothifying upperright")
        mesh4 = lib.empty_cloth()
        nodes4 = clothify_stroke(mesh4, 'upperright')

        print("creating frames...")


        node_count = lib.mesh_node_count(mesh1)
        min_height_min_width_node = lib.mesh_node_position(mesh1, node_count-1)
        node_count = lib.mesh_node_count(mesh2)
        max_height_node = lib.mesh_node_position(mesh2, node_count-1)
        node_count = lib.mesh_node_count(mesh3)
        max_width_node = lib.mesh_node_position(mesh3, node_count-1)
        #node_count = lib.mesh_node_count(mesh4)
        #max_width_node = lib.mesh_node_position(mesh4, node_count-1)
        
        mesh5 = lib.empty_cloth()
        
        x_positions, y_positions = generate_grid_coords(16, 9, min_height_min_width_node, max_height_node, max_width_node) 
        mesh5_nodes_ordered, (bl_corner, tr_corner, tl_corner, br_corner, middle), edges_dict = generate_mesh_grid(mesh5, x_positions, y_positions, 0.06 )
        #bl_corner, tr_corner, tl_corner, br_corner = generate_mesh_grid(mesh5, range(10), range(10), 0.1)
        
        #lib.add_connector(mesh1, nodes1[-1], bl_corner, 0.5)
        #lib.add_connector(mesh2, nodes2[-1], br_corner, 0.5)
        #lib.add_connector(mesh3, nodes3[-1], tl_corner, 0.5)
        #lib.add_connector(mesh4, nodes4[-1], tr_corner, 0.5)
        
        lib.add_connector(mesh5, bl_corner, nodes1[-1], 0.01, 0.5)
        lib.add_connector(mesh5, br_corner, nodes2[-1], 0.01, 0.5)
        lib.add_connector(mesh5, tl_corner, nodes3[-1], 0.01, 0.5)
        lib.add_connector(mesh5, tr_corner, nodes4[-1], 0.01, 0.5)
        
        lib.add_connector(mesh5, bl_corner, tl_corner, 1.25, 0.5)
        lib.add_connector(mesh5, tr_corner, br_corner, 0.5, 0.5)
        lib.add_connector(mesh5, tl_corner, br_corner, 0.75, 0.5)
        lib.add_connector(mesh5, tr_corner, bl_corner, 2, 0.5)
        #lib.add_connector(mesh5, bl_corner, middle, 2, 0.5)
        #lib.add_connector(mesh5, br_corner, middle, 2, 0.5)
        #lib.add_connector(mesh5, tl_corner, middle, 2, 0.5)
        #lib.add_connector(mesh5, tr_corner, middle, 2, 0.5)
                
        bpy.ops.object.mode_set(mode='OBJECT')
        gp_layer = init_grease_pencil()

        #create_frames(mesh1, gp_layer)
        #create_frames(mesh2, gp_layer)
        #create_frames(mesh3, gp_layer)
        #create_frames(mesh4, gp_layer)
        #create_frames(mesh5, gp_layer)
        
        fold_in_frames([mesh1, mesh2, mesh3, mesh4, mesh5], gp_layer, [lambda gp_frame: draw_mesh_grid(mesh5, gp_frame, (16,9))])
        
        return {'FINISHED'}
    
def generate_grid_coords(width, height, both_min, max_height, max_width):
    
    height_diff = (max_height.y - both_min.y)/height
    y_positions = [both_min.y + height_diff * i * 2 for i in range(height)]

    width_diff = (max_width.x - both_min.x)/width
    x_positions = [both_min.x + width_diff * i * 2 for i in range(width)]
    
    return (x_positions, y_positions)
            

def generate_mesh_grid(mesh, x_positions, y_positions, spacing):
    verlet_nodes = []
    
    left_edge_nodes = []
    right_edge_nodes = []
    upper_edge_nodes = []
    lower_edge_nodes = []
    
    for y_idx, y in enumerate(y_positions):
        for x_idx, x in enumerate(x_positions):
            verlet_node = lib.create_verlet_node(x, y)
            
            #name = ffi.new('char[]', bytes("gravity", 'utf-8'))
            #lib.add_gravity(mesh, verlet_node, name, 0.033, -98)
            name = ffi.new('char[]', bytes("gravity", 'utf-8'))
            lib.add_bound_gravity(mesh, verlet_node, name, 0.005, -98, (y-1)*3)
            
            name = ffi.new('char[]', bytes("wind", 'utf-8'))
            lib.add_wind(mesh, verlet_node, name, 0.125, 0.0005 )

            #name = ffi.new('char[]', bytes("ground", 'utf-8'))
            #lib.add_ground_boundary(mesh, verlet_node, name, GROUND_BOUND)

            modified_spacing = abs(spacing * (x_idx + 0.1 - len(x_positions))) if (x_idx > len(x_positions)/2) else abs(0.1 + spacing * x_idx) # if above half index, subtract max index and get abs
            if x_idx != 0:
                lib.add_connector(mesh, verlet_node, verlet_nodes[-1], modified_spacing, 0.5)
            if y_idx != 0:
                lib.add_connector(mesh, verlet_node, verlet_nodes[x_idx+(y_idx-1)*len(x_positions)], modified_spacing, 0.5)
                
            # add node to mesh
            lib.add_node_to_mesh(mesh, verlet_node)
            
            verlet_nodes.append(verlet_node)
            
            # get corners
            if y_idx == 0 and x_idx == 0:
                bottom_left = verlet_node
            if len(x_positions) - 1 == x_idx and len(y_positions)-1 == y_idx:
                top_right = verlet_node
            if y_idx == (len(y_positions) - 1) and x_idx == 0:
                top_left = verlet_node
            if x_idx == (len(x_positions) - 1) and y_idx == 0:
                bottom_right = verlet_node
                
            #get edges
            if x_idx == 0:
                left_edge_nodes.append(verlet_node)
            if len(x_positions) - 1 == x_idx:
                right_edge_nodes.append(verlet_node)
            if y_idx == (len(y_positions) - 1):
                upper_edge_nodes.append(verlet_node)
            if y_idx == 0:
                lower_edge_nodes.append(verlet_node)
                
    
    middle = verlet_nodes[int(len(verlet_nodes)/2)]
                
    return (verlet_nodes, (bottom_left, top_right, top_left, bottom_right, middle),
            {'upper_edge': upper_edge_nodes,
            'lower_edge': lower_edge_nodes,
            'left_edge': left_edge_nodes,
            'right_edge': right_edge_nodes,
            })
        

def create_frames(mesh, gp_layer, callbacks = []):

    NUM_FRAMES = 600
    FRAMES_SPACING=1

    
    for frame in range(NUM_FRAMES):
        if len(list(gp_layer.frames)) <= frame*FRAMES_SPACING:
            gp_frame = gp_layer.frames.new(frame*FRAMES_SPACING)
        else:
            gp_frame = gp_layer.frames[frame*FRAMES_SPACING]
        #node_count = lib.mesh_node_count(mesh)
        #node_positions = [lib.mesh_node_position(mesh, i) for i in range(node_count)]
        #print(node_positions)

        inter_con_count = lib.get_cloth_interconnector_count(mesh)
        constraint_counts = [lib.get_interconnector_constraint_count(mesh, index) for index in range(inter_con_count)]
        
        #if frame == 100:
        #    name = ffi.new('char[]', bytes("wind", 'utf-8'))
        #    lib.remove_constraint_by_id(mesh, name)

        #if frame == 6:
        #    name = ffi.new('char[]', bytes('gravity', 'utf-8'))
        #    lib.remove_constraint_by_id(mesh, name)

        connector_positions = []
        for inter_index in range(inter_con_count):
            connector_positions += [lib.get_interconnector_constraint(mesh, inter_index, index) for index in range(constraint_counts[inter_index])]
        
        #for node in connector_positions:
        #    draw_line(gp_frame, (node.x, 0, node.y), (node.dx, 0, node.dy))
        nodes = [(node.x, 0, node.y) for node in connector_positions]
        draw_stroke(gp_frame, nodes)
        
        for callback in callbacks:
            callback(gp_frame)
            
        lib.update_cloth_mesh(mesh, 0.005, 4)

def fold_in_frames(meshes, gp_layer, callbacks = []):

    NUM_FRAMES = 600
    FRAMES_SPACING=1

    
    for frame in range(NUM_FRAMES):
        for mesh in meshes:
            if len(list(gp_layer.frames)) <= frame*FRAMES_SPACING:
                gp_frame = gp_layer.frames.new(frame*FRAMES_SPACING)
            else:
                gp_frame = gp_layer.frames[frame*FRAMES_SPACING]

            inter_con_count = lib.get_cloth_interconnector_count(mesh)
            constraint_counts = [lib.get_interconnector_constraint_count(mesh, index) for index in range(inter_con_count)]

            connector_positions = []
            for inter_index in range(inter_con_count):
                connector_positions += [lib.get_interconnector_constraint(mesh, inter_index, index) for index in range(constraint_counts[inter_index])]
            
            #nodes = [(node.dx, 0, node.dy) for node in connector_positions]
            #draw_stroke(gp_frame, nodes)
            for node in connector_positions:
                draw_line(gp_frame, (node.x, 0, node.y), (node.dx, 0, node.dy))

            for callback in callbacks:
                callback(gp_frame)

            lib.update_cloth_mesh(mesh, 0.005, 3)


class InvalidMeshDimensions(Exception):
    """
    """
    pass

def draw_mesh_grid(mesh, gp_frame, mesh_dim):
    max_node_count = lib.mesh_node_count(mesh)
    for y_idx in reversed(range(mesh_dim[1])):
        for x_idx in reversed(range(mesh_dim[0])):
            if x_idx != 0 and y_idx !=0:
                if x_idx+(y_idx)*mesh_dim[0] > max_node_count:
                    raise InvalidMeshDimensions
                br_node = lib.mesh_node_position(mesh, x_idx+(y_idx)*mesh_dim[0]) #mesh_nodes[x_idx+(y_idx)*mesh_dim[0]]
                br_node_co = [br_node.x, 0, br_node.y]
                tr_node = lib.mesh_node_position(mesh, x_idx+(y_idx-1)*mesh_dim[0]) #mesh_nodes[x_idx+(y_idx-1)*mesh_dim[0]]
                tr_node_co = [tr_node.x, 0, tr_node.y]
                tl_node = lib.mesh_node_position(mesh, x_idx-1+(y_idx-1)*mesh_dim[0]) #mesh_nodes[x_idx-1+(y_idx-1)*mesh_dim[0]]
                tl_node_co = [tl_node.x, 0, tl_node.y]
                bl_node = lib.mesh_node_position(mesh, x_idx-1+(y_idx)*mesh_dim[0]) #mesh_nodes[x_idx-1+(y_idx)*mesh_dim[0]]
                bl_node_co = [bl_node.x, 0, bl_node.y]
                            
                draw_stroke(gp_frame, [tl_node_co, br_node_co, bl_node_co, tr_node_co], cyclic = True, material_index = 2)


def clothify_stroke(mesh, vertex_group_name):
    o = bpy.context.active_object
    idx = o.vertex_groups[vertex_group_name].index
    
    bpy.context.active_object.vertex_groups.active_index = idx
    bpy.ops.object.mode_set(mode='EDIT_GPENCIL')

    bpy.ops.gpencil.vertex_group_select()
    
    #vs = [ v.active_frame.strokes for layer_name, v in o.data.layers if idx in [ vg.group for vg in v.groups ] ]
    
    for layer_name, layer in o.data.layers.items():
        strokes = [stroke for stroke in layer.active_frame.strokes]
        selected_points = []
        for stroke in strokes:
            selected_points += [point for point in stroke.points if point.select]
    
    # create verlet nodes from selected
    verlet_nodes = []
    last_coords = None
    for vertex in selected_points:
        #print(vertex.as_pointer())
        #print(vertex.co)
        #print(vertex.select)
        if verlet_nodes:
            verlet_node = lib.create_verlet_node(vertex.co[0], vertex.co[2])
            # constrain new node to last node
            #lib.add_connector(mesh, verlet_nodes[-1], verlet_node, 0.5)
            lib.add_connector(mesh, verlet_node, 
                              verlet_nodes[-1],
                              distance_betwixt(last_coords, (vertex.co[0], vertex.co[2]))/3,
                              0.75)
            # add gravity to nodes
            name = ffi.new('char[]', bytes("gravity", 'utf-8'))
            lib.add_gravity(mesh, verlet_node, name, 0.005, -98)
            #name = ffi.new('char[]', bytes("gravity", 'utf-8'))
            #lib.add_bound_gravity(mesh, verlet_node, name, 0.033, -98, GROUND_BOUND)
            # add ground
            #name = ffi.new('char[]', bytes("ground", 'utf-8'))
            #lib.add_ground_boundary(mesh, verlet_node, name, GROUND_BOUND)
            
            #name = ffi.new('char[]', bytes("wind", 'utf-8'))
            #lib.add_wind(mesh, verlet_node, name, 0.1, 0)
        else:
            print("creating pinned node")
            verlet_node = lib.create_pinned_verlet_node(vertex.co[0],vertex.co[2])
        
        # add node to mesh
        lib.add_node_to_mesh(mesh, verlet_node)

        verlet_nodes.append(verlet_node)
        last_coords = (vertex.co[0], vertex.co[2])

    
    bpy.ops.gpencil.vertex_group_deselect()
    return verlet_nodes


def distance_betwixt(pos1, pos2):
    diffx = pos1[0] - pos2[0]
    diffz = pos1[1] - pos2[1]
    return math.sqrt(diffx*diffx+diffz*diffz)


classes = (
    NAC_MeshList,
    NAC_constraints,
    addMesh,
    Stringify
)


# to get vertice groups: 
# o = bpy.context.object
# vs = [ v for v in o.data.vertices if vg_idx in [ vg.group for vg in v.groups ] ]

# get all vertices in a layer
# stroke_list = bpy.context.scene.objects['Stroke'].data.layers['Lines'].active_frame.strokes
# for stroke in stroke_list:
#     for point in stroke.points:
#         print(point.co)
#         print(point.select)


if __name__ == "__main__":  # only for live edit.
    from bpy.utils import register_class
    for cls in classes:
        register_class(cls)
    bpy.ops.object.mode_set(mode='OBJECT')
    #mesh = lib.empty_cloth()
    #print("Clothifying lowerleft")
    #clothify_stroke(mesh, 'lowerleft')
    #print("Clothifying lowerright")
    #clothify_stroke(mesh, 'lowerright')
    #print("Clothifying upperleft")
    #clothify_stroke(mesh, 'upperleft')
    #print("Clothifying upperright")
    #clothify_stroke(mesh, 'upperright')
    #print("creating frames...")
    #create_frames(mesh)
    ##lib.mesh_free(mesh)