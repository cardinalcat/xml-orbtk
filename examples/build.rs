use xml_orbtk::builder::Builder;

fn main() {
    let mut builder = Builder::new("res/build.xml").unwrap();
    builder.parse();
    builder.build_app();
    builder.run();
}
