use xml_orbtk::builder::Builder;

fn main() {
    println!("tuple: {}", stringify!((u32, u32)));
    println!("vector: {}", stringify!(&[u32]));
    let mut builder = Builder::new("res/build.xml").unwrap();
    builder.parse();
    builder.build_app();
    builder.run();
}
