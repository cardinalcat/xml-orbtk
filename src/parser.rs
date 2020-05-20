use orbtk::{
    prelude::*,
    shell::{ShellRunner, WindowBuilder, WindowShell},
    tree::*,
    utils::{Point, Rectangle},
};
use ego_tree::iter::FirstChildren;
use scraper::{element_ref::ElementRef, node::Element, Html, Selector, node::Node};
use std::str::FromStr;
use std::io::Error;
pub struct TypeConverter {
    data: String,
    datatype: &'static str,
}
impl TypeConverter {
    pub fn new(data: &str, datatype: &'static str) -> Self {
        println!("datatype: {}", datatype);
        Self {
            data: data.to_string(),
            datatype,
        }
    }
    pub fn get_output<T>(&self) -> T
        where
            T: std::str::FromStr,
            <T as std::str::FromStr>::Err: Debug
    {
        let ret: T = self.data.parse().unwrap();
        ret
    }
}
#[derive(Debug, Clone)]
pub struct WindowParser {
    pub index: usize,
    pub id: Option<String>,
    pub text: String,
}
pub trait Parser {
    fn parse_attributes(self, element: &Element) -> Self;
}
#[macro_export]
macro_rules! generate_attribute {
    ($widgetname:ty, $( $param:ty => $callback:tt ),* ) => {
           impl Parser for $widgetname {
            fn parse_attributes(mut self, element: &Element) -> Self{
                $(
                    match element.attr(stringify!($callback)){
                        Some(val) => {
                            let converter = TypeConverter::new(val, stringify!($param));
                            let value: $param = converter.get_output();

                            self = self.$callback(value);
                        },
                        None => (),
                    }
                )*
                self
            }
        }
    }
}
#[macro_export]
macro_rules! generate_enum {
    ($name:ident, $( $vari:ident => $type:ty ),*) => {
        pub enum $name {
            $(
                $vari($type),
            )*
        }
        impl $name {
            pub fn build(self, ctx: &mut BuildContext) -> Option<Entity>{
                match self{
                    $(
                        $name::$vari(val) => {
                            Some(val.build(ctx))
                        },
                    )*
                    _ => None,
                }
            }
            pub fn child(self, child: Entity) -> Option<Self>{
                match self{
                    $(
                        $name::$vari(val) => {
                            Some($name::$vari(val.child(child)))
                        },
                    )*
                    _ => None,
                }
            }
        }
        impl Parser for $name {
            fn parse_attributes(self, element: &Element) -> Self{
                match self{
                    $(
                        $name::$vari(val) => {
                            $name::$vari(val.parse_attributes(element))
                        },
                    )*
                    _ => panic!("unhandled widget type"),
                }
            }
        }
        impl FromStr for $name {
            type Err = std::io::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err>{
                //let matcher = 
                match s {
                    $(
                        stringify!($vari) => Ok($name::$vari(<$type>::create())),
                    )*
                    _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "couldn't parse widget type")),
                }
            }
        }
    }
}
generate_attribute! (Window,
    f64 => width, 
    f64 => height, 
    bool => borderless, 
    String => title,
    String => id,
    f64 => max_height,
    f64 => max_width,
    f32 => opacity,
    bool => resizeable,
    bool => always_on_top
);
generate_attribute! (Button,
    String => text
);
generate_attribute! (TextBox,
    String => text
);
generate_enum! (XmlElement, TextBox => TextBox, Window => Window, Button => Button);
impl WindowParser {
    pub fn new(text: String, id: Option<String>, index: usize) -> Self {
        Self { text, id, index }
    }
    pub fn build(&mut self, ctx: &mut BuildContext) -> Entity {
        let html = Html::parse_fragment(&self.text);
        let selector = Selector::parse("window").unwrap();
        if let Some(element) = html.select(&selector).next() {
            Self::build_elements(element, ctx)
        } else {
            panic!("no windows found");
        }
    }
    pub fn build_elements(element: ElementRef, ctx: &mut BuildContext) -> Entity {
        let value = element.value();
        match value.name() {
            "window" => {
                if element.has_children(){
                    Window::create().parse_attributes(&value).child(Self::handle_children(element.first_children(), ctx)).build(ctx)
                }
                else {
                    Window::create().parse_attributes(&value).build(ctx)
                }
            },
            _ => panic!("window should be top level widget"),
        }
    }
    fn handle_children(mut children: FirstChildren<Node>, ctx: &mut BuildContext) -> Entity{
        while let Some(html_node) = children.next(){
            if let Node::Element(elem) = html_node.value(){
                let xmlelement = XmlElement::from_str(elem.name());
                let mut item = xmlelement.unwrap();
                let ent = if html_node.has_children(){
                    item.child(Self::handle_children(html_node.first_children(),ctx)).unwrap().build(ctx)
                }else{
                    item.build(ctx)
                };
            }
        }
        Button::create().build(ctx)
    }
}
