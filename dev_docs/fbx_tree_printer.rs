fn print_children(depth: usize, children: Children) {
    let show_depth = depth * 3;
    for child in children {
        print!("{space:>depth$}", space = "", depth = show_depth);
        if child.name().len() > 1 {
            print!("{name} ", name = child.name(),);
        }
        let attr_display = |att: &AttributeValue| match att {
            AttributeValue::Bool(v) => format!("{v}"),
            AttributeValue::I16(v) => format!("{v}"),
            AttributeValue::I32(v) => format!("{v}"),
            AttributeValue::I64(v) => format!("{v}"),
            AttributeValue::F32(v) => format!("{v}"),
            AttributeValue::F64(v) => format!("{v}"),
            AttributeValue::ArrBool(_) => "[bool]".to_owned(),
            AttributeValue::ArrI32(_) => "[i32]".to_owned(),
            AttributeValue::ArrI64(_) => "[i64]".to_owned(),
            AttributeValue::ArrF32(_) => "[f32]".to_owned(),
            AttributeValue::ArrF64(_) => "[f64]".to_owned(),
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Binary(_) => "[u8]".to_owned(),
        };
        print!("[");
        for (i, attr) in child.attributes().iter().map(attr_display).enumerate() {
            // if matches!(i, 1 | 2 | 3) {
            //     continue;
            // }
            if i == 0 {
                print!("{attr}: ");
            } else {
                print!("{attr}, ");
            }
        }
        println!("]");
        if child.children().next().is_some() {
            println!("{:>depth$}{{", "", depth = show_depth);
        }
        print_children(depth + 1, child.children());
        if child.children().next().is_some() {
            println!("{:>depth$}}}", "", depth = show_depth);
        }
    }
}
