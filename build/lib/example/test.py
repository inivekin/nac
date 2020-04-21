from example._native import ffi, lib


def test():
    cloth_ptr = lib.get_cloth_mesh(5,10,5)
    print(lib.mesh_node_count(cloth_ptr))
    lib.mesh_free(cloth_ptr)

    lib.update_cloth_mesh(cloth_ptr, 0.0016, 3)
    import time
    while True:
        positions = lib.get_cloth_mesh_positions(cloth_ptr)
        print(positions)
        time.sleep(0.2)

if __name__ == '__main__':
    test()
