// Adopted from fbxcel-dom example.
// Scan given fbx file to get useful info.

fn main() {
    env_logger::init();

    let path = match std::env::args_os().nth(1) {
        Some(v) => std::path::PathBuf::from(v),
        None => {
            eprintln!("Usage: load: <FBX_FILE>");

            std::process::exit(1);
        }
    };

    let file = std::fs::File::open(path).expect("Failed to open file");
    let reader = std::io::BufReader::new(file);

    if let fbxcel_dom::any::AnyDocument::V7400(ver, doc) =
        fbxcel_dom::any::AnyDocument::from_seekable_reader(reader).expect("Failed to load document")
    {
        println!("Loaded FBX with version = {:?}\n", ver);

        doc.scenes().for_each(|scene| {
            if scene.name().is_some() {
                println!("Scene: {:#?}", scene.name());
            } else {
                println!("Scene: (no name)");
            }

            println!("  Class - {:#?}", scene.class());
            println!("  Subclass - {:#?}", scene.subclass());
            println!("  Node ID - {:#?}", scene.object_node_id());
            println!("  Root ID - {:#?}", scene.root_object_id());
            println!("  Object ID - {:#?}", scene.object_id());
            println!("  Object type - {:#?}", scene.get_typed());
        });
    } else {
        panic!("FBX version unsupported by this example")
    }
}
