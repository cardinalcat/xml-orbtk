use crate::parser::WindowParser;
use orbtk::{
    prelude::*,
    shell::{WindowBuilder},
    tree::*,
    utils::{Point, Rectangle},
};
use scraper::{Html, Selector};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    rc::Rc,
};
use std::iter::FromIterator;
pub struct Builder {
    app: Application,
    src: String,
    windows: Vec<WindowParser>,
    name: Box<str>,
}
impl Builder {
    pub fn new(file_name: &str) -> io::Result<Self> {
        Self::from_name(file_name, Box::default())
    }
    pub fn from_name(file_name: &str, name: Box<str>) -> io::Result<Self> {
        let mut src = String::new();
        let mut file = File::open(file_name)?;
        file.read_to_string(&mut src)?;
        let app = Application::from_name(name.clone());
        Ok(Self {
            app,
            src,
            windows: Vec::new(),
            name: name,
        })
    }
    pub fn add_css_path(mut self, path: &str) -> Self{
        let iter = self.windows.clone();
        self.windows.clear();
        self.add_all(iter.into_iter(), path)
    }
    fn add_all(mut self, mut windows: std::vec::IntoIter<WindowParser>, path: &str) -> Self{
        match windows.next() {
            Some(mut window) => {
                self.windows.push(window.add_css_path(path));
                self.add_all(windows, path)
            },
            None => self,
        }
    }
    pub fn parse(mut self) -> Self {
        let fragment = Html::parse_fragment(&self.src);
        let selector = Selector::parse("window").unwrap();
        for element in fragment.select(&selector) {
            /*println!(
                "attributes: {:?}, children: {:?}",
                element.value().attrs,
                Html::parse_fragment(&element.inner_html())
            );*/
            self.windows.push(WindowParser::new(
                element.html(),
                None,
                0,
            ));
        }
        self
    }
    ///this builds the main window, note this code is taken from the application struct's implementation in orbtk
    pub fn add_window<F: Fn(&mut BuildContext) -> Entity + 'static>(
        mut self,
        create_fn: F,
    ) -> Self{
        self.app = self.app.window(create_fn);
        self
    }
    pub fn build_app(mut self) -> Self{
        let iter = self.windows.clone();
        //let mut newvec = Vec::from_iter(iter);
        self.build_rec(iter.into_iter())
    }
    fn build_rec(mut self, mut windows: std::vec::IntoIter<WindowParser>) -> Self{
        match windows.next() {
            Some(mut window) => {
                let builder = self.add_window(move |ctx|{
                    window.build(ctx)
                });
                builder.build_rec(windows)
            },
            None => self,
        }
    }
    pub fn run(mut self) {
        self.app.run();
    }
}
