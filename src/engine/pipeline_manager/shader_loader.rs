use vulkano_shaders;

vulkano_shaders::shader! {
    shaders: {
        normal_vert: {
            ty: "vertex",
            path: "shaders/debug/normal.vert"
        },
        normal_frag: {
            ty: "fragment",
            path: "shaders/debug/normal.frag"
        },


        depth_vert: {
            ty: "vertex",
            path: "shaders/debug/depth.vert"
        },
        depth_frag: {
            ty: "fragment",
            path: "shaders/debug/depth.frag"
        },


        mesh_view_vert: {
            ty: "vertex",
            path: "shaders/debug/mesh_view.vert"
        },
        mesh_view_frag: {
            ty: "fragment",
            path: "shaders/debug/mesh_view.frag"
        }
    }
}
