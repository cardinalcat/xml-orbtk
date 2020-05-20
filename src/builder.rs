use crate::parser::WindowParser;
use orbtk::{
    prelude::*,
    shell::{ShellRunner, WindowBuilder, WindowShell},
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
pub struct Builder {
    runners: Vec<ShellRunner<WindowAdapter>>,
    src: String,
    windows: Vec<Rc<RefCell<WindowParser>>>,
    name: Box<str>,
}
impl Builder {
    pub fn new(file_name: &str) -> io::Result<Self> {
        Self::from_name(file_name, Box::default())
    }
    pub fn from_name(file_name: &str, name: impl Into<Box<str>>) -> io::Result<Self> {
        let mut src = String::new();
        let mut file = File::open(file_name)?;
        file.read_to_string(&mut src)?;
        Ok(Self {
            runners: Vec::new(),
            src,
            windows: Vec::new(),
            name: name.into(),
        })
    }
    pub fn get_window_shell<'a>(
        &mut self,
        index: usize,
    ) -> Option<Rc<RefCell<WindowShell<WindowAdapter>>>> {
        match self.runners.get(index) {
            Some(runner) => Some(runner.window_shell.clone()),
            None => None,
        }
    }
    pub fn parse(&mut self) {
        let fragment = Html::parse_fragment(&self.src);
        let selector = Selector::parse("window").unwrap();
        for element in fragment.select(&selector) {
            /*println!(
                "attributes: {:?}, children: {:?}",
                element.value().attrs,
                Html::parse_fragment(&element.inner_html())
            );*/
            self.windows.push(Rc::new(RefCell::new(WindowParser::new(
                element.html(),
                None,
                0,
            ))));
        }
    }
    ///this builds the main window, note this code is taken from the application struct's implementation in orbtk
    pub fn add_window<F: Fn(&mut BuildContext) -> Entity + 'static>(
        name: Box<str>,
        create_fn: F,
    ) -> ShellRunner<WindowAdapter> {
        let mut world = World::from_stores(Tree::default(), StringComponentStore::default());

        let render_objects = Rc::new(RefCell::new(BTreeMap::new()));
        let layouts = Rc::new(RefCell::new(BTreeMap::new()));
        let handlers = Rc::new(RefCell::new(BTreeMap::new()));
        let states = Rc::new(RefCell::new(BTreeMap::new()));
        let registry = Rc::new(RefCell::new(Registry::new()));

        if name.is_empty() {
            registry
                .borrow_mut()
                .register("settings", Settings::default());
        } else {
            registry
                .borrow_mut()
                .register("settings", Settings::new(&*name));
        };
        let window = {
            let overlay = Overlay::create().build(&mut BuildContext::new(
                world.entity_component_manager(),
                &render_objects,
                &mut layouts.borrow_mut(),
                &mut handlers.borrow_mut(),
                &mut states.borrow_mut(),
                &mut orbtk::theme::default_theme(),
            ));

            {
                let tree: &mut Tree = world.entity_component_manager().entity_store_mut();
                tree.set_overlay(overlay);
            }

            let window = create_fn(&mut BuildContext::new(
                world.entity_component_manager(),
                &render_objects,
                &mut layouts.borrow_mut(),
                &mut handlers.borrow_mut(),
                &mut states.borrow_mut(),
                &mut orbtk::theme::default_theme(),
            ));

            {
                let tree: &mut Tree = world.entity_component_manager().entity_store_mut();
                tree.set_root(window);
            }

            window
        };
        let title = world
            .entity_component_manager()
            .component_store()
            .get::<String>("title", window)
            .unwrap()
            .clone();
        let borderless = *world
            .entity_component_manager()
            .component_store()
            .get::<bool>("borderless", window)
            .unwrap();
        let resizeable = *world
            .entity_component_manager()
            .component_store()
            .get::<bool>("resizeable", window)
            .unwrap();
        let always_on_top = *world
            .entity_component_manager()
            .component_store()
            .get::<bool>("always_on_top", window)
            .unwrap();
        let position = *world
            .entity_component_manager()
            .component_store()
            .get::<Point>("position", window)
            .unwrap();
        let constraint = *world
            .entity_component_manager()
            .component_store()
            .get::<Constraint>("constraint", window)
            .unwrap();

        world
            .entity_component_manager()
            .component_store_mut()
            .register("global", window, Global::default());
        world
            .entity_component_manager()
            .component_store_mut()
            .register("global", window, Global::default());
        world
            .entity_component_manager()
            .component_store_mut()
            .register(
                "bounds",
                window,
                Rectangle::from((0.0, 0.0, constraint.width(), constraint.height())),
            );

        let window_shell = Rc::new(RefCell::new(
            WindowBuilder::new(WindowAdapter {
                root: window,
                render_objects: render_objects.clone(),
                layouts: layouts.clone(),
                handlers: handlers.clone(),
                states: states.clone(),
                ..Default::default()
            })
            .title(&(title)[..])
            .bounds(Rectangle::from((
                position.x,
                position.y,
                constraint.width(),
                constraint.height(),
            )))
            .borderless(borderless)
            .resizeable(resizeable)
            .always_on_top(always_on_top)
            .build(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        window_shell
            .borrow_mut()
            .render_context_2_d()
            .register_font("Roboto Regular", orbtk::theme::fonts::ROBOTO_REGULAR_FONT);

        #[cfg(not(target_arch = "wasm32"))]
        window_shell
            .borrow_mut()
            .render_context_2_d()
            .register_font("Roboto Medium", orbtk::theme::fonts::ROBOTO_MEDIUM_FONT);

        #[cfg(not(target_arch = "wasm32"))]
        window_shell
            .borrow_mut()
            .render_context_2_d()
            .register_font(
                "Material Icons",
                orbtk::theme::fonts::MATERIAL_ICONS_REGULAR_FONT,
            );

        world.register_init_system(InitSystem {
            shell: window_shell.clone(),
            layouts: layouts.clone(),
            render_objects: render_objects.clone(),
            handlers: handlers.clone(),
            states: states.clone(),
            registry: registry.clone(),
        });

        world.register_cleanup_system(CleanupSystem {
            shell: window_shell.clone(),
            layouts: layouts.clone(),
            render_objects: render_objects.clone(),
            handlers: handlers.clone(),
            states: states.clone(),
            registry: registry.clone(),
        });

        world
            .create_system(EventStateSystem {
                shell: window_shell.clone(),
                handlers: handlers.clone(),
                mouse_down_nodes: RefCell::new(vec![]),
                render_objects: render_objects.clone(),
                states: states.clone(),
                layouts: layouts.clone(),
                registry: registry.clone(),
            })
            .with_priority(0)
            .build();

        world
            .create_system(LayoutSystem {
                shell: window_shell.clone(),
                layouts: layouts.clone(),
            })
            .with_priority(1)
            .build();

        world
            .create_system(PostLayoutStateSystem {
                shell: window_shell.clone(),
                layouts: layouts.clone(),
                render_objects: render_objects.clone(),
                handlers: handlers.clone(),
                states: states.clone(),
                registry: registry.clone(),
            })
            .with_priority(2)
            .build();

        world
            .create_system(RenderSystem {
                shell: window_shell.clone(),
                layouts: layouts.clone(),
                render_objects: render_objects.clone(),
                handlers: handlers.clone(),
                states: states.clone(),
            })
            .with_priority(3)
            .build();

        //self.runners.push(
        ShellRunner {
            updater: Box::new(WorldWrapper { world }),
            window_shell,
        }
    }
    pub fn build_app(&mut self) {
        let mut runners = Vec::new();
        let tmpname = self.name.clone();
        for win in self.windows.iter_mut() {
            let winpar = win.clone();
            runners.push(Builder::add_window(tmpname.clone(), move |ctx| {
                winpar.borrow_mut().build(ctx)
            }));
        }
        self.runners.append(&mut runners);
    }
    pub fn run(&mut self) {
        for mut runner in self.runners.iter_mut() {
            runner.run();
        }
    }
}
