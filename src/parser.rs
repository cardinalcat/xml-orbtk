use orbtk::{
    prelude::*,
    shell::{ShellRunner, WindowBuilder, WindowShell},
    tree::*,
    utils::{Point, Rectangle},
};
use ego_tree::NodeRef;
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
            pub fn build(self, ctx: &mut BuildContext) -> Entity{
                match self{
                    $(
                        $name::$vari(val) => {
                            val.build(ctx)
                        },
                    )*
                    _ => panic!("unhandled widget type"),
                }
            }
            pub fn child(self, child: Entity) -> Self{
                match self{
                    $(
                        $name::$vari(val) => {
                            $name::$vari(val.child(child))
                        },
                    )*
                    _ => panic!("unhandled widget type"),
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
    f64 => min_width,
    f64 => min_height,
    f32 => opacity,
    bool => resizeable,
    bool => always_on_top
);
generate_attribute! (Button,
    String => text
);
generate_attribute! (TextBox,
    String => text,
    f64 => width,
    f64 => height,
    f64 => max_width,
    f64 => max_height,
    f64 => min_width,
    f64 => min_height
);
generate_enum! (XmlElement, textbox => TextBox, Window => Window, Button => Button);
impl WindowParser {
    pub fn new(text: String, id: Option<String>, index: usize) -> Self {
        //let map: HashMap<&str, Fn(FromStr) -> Widget> = 
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
                let recelement = XmlElement::from_str("Window").unwrap();
                let (_, elem) = Self::handle_children(element.first_child().unwrap(),ctx,recelement);
                elem.unwrap().build(ctx)
            },
            _ => panic!("window should be top level widget"),
        }
    }
    fn handle_children(mut noderef: NodeRef<Node>, ctx: &mut BuildContext, mut parent: XmlElement) -> (Option<Entity>, Option<XmlElement>){
        if let Node::Element(html_element) = noderef.value(){     
            println!("element name: {}", html_element.name());
            let mut xmlelement = XmlElement::from_str(html_element.name()).unwrap().parse_attributes(&html_element);
            let (curent, _): (Option<Entity>, Option<XmlElement>) = match noderef.first_child(){
                Some(child) => {
                    let (_, xmlelem) = Self::handle_children(child, ctx, xmlelement);
                    let xmlelement = xmlelem.unwrap();
                    //xmlelement = xmlelement.child(childent);
                    (Some(xmlelement.build(ctx)), None)
                },
                None => {
                    (Some(xmlelement.build(ctx)), None)
                },
            };
            match noderef.next_sibling(){
                Some(sibiling) => {
                    parent = parent.child(curent.unwrap());
                    Self::handle_children(sibiling, ctx, parent)
                },
                None => {
                    parent = parent.child(curent.unwrap());
                    (curent, Some(parent))
                },
            }
        }else{
            match noderef.next_sibling(){
                Some(sibiling) => {
                    Self::handle_children(sibiling, ctx, parent)
                },
                None => {
                    (None, Some(parent))
                },
            }
        }
    }

}
/*
if let Node::Element(elem) = html_node.value(){
                let xmlelement = XmlElement::from_str(elem.name());
                let mut item = xmlelement.unwrap().parse_attributes(&elem);
                let ent = if html_node.has_children(){
                    let (ent, item) = Self::handle_children(html_node.first_child().unwrap(),ctx, item);
                    item.child(ent).unwrap().build(ctx).unwrap()
                }else{
                    item.build(ctx).unwrap()
                };
                return ent;
            }*/