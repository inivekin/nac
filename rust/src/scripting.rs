use crate::nac::{Mesh};
use crate::verlet::{Verlet};
use crate::cloth::{Cloth};

use libc;


// cloth wrapper fn new_cloth(_py: Python, )
//fn new_cloth(_: Python, height: u8, width: u8, spacing: u8) -> PyResult<Mesh<Verlet>> {
    //Ok(Cloth::new_cloth(height, width, height))
//}

//#[no_mangle]
//pub extern fn new_cloth(height: libc::u8, width: libc::u8, spacing: libc::u8) -> Mesh<Verlet> {
    //Ok(Cloth::new_cloth(height, width, height))
//}

//py_module_initializer!(libnac, initlibnac, PyInit_libnac, |py, m| {
    //(m.add(py, "__doc__", "Nodes and connectors model"))?;
    //(m.add(py, "new_cloth", py_fn!(py, new_cloth(height: u8, width: u8, spacing: u8))))?;
    //Ok(())
//});
