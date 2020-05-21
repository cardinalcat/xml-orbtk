use xml_orbtk::builder::Builder;

fn main() {
    let mut builder = Builder::new("res/build.xml").unwrap().parse().add_css_path("res/build.css").build_app().run();
}