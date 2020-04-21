from example._native import ffi, lib


def test():
    cloth_ptr = lib.get_cloth_mesh(5,5,5)
    print(lib.mesh_node_count(cloth_ptr))
    lib.mesh_free(cloth_ptr)

if __name__ == '__main__':
    test()
