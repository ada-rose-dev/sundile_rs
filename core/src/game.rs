use sundile_assets::*;
use sundile_common::*;
use sundile_graphics::*;

use crate::defaults::default_scene;
use crate::defaults::load_default_assets;
use crate::renderer::*;
use crate::renderer2d::*;
use crate::SceneBuilder;
use crate::SceneFn;
use crate::SceneMap;

pub struct Game {
    pub renderer: Renderer,
    pub renderer2d: Renderer2d,
    pub paused: bool,
    pub assets: AssetTypeMap,
    scenes: SceneMap, //TODO: Possibly move this outside of Game struct so DebugGui has ability to change scenes?
    scene_initialized: bool,
}

impl Game {
    pub fn new(
        render_target: &RenderTarget,
        mut assets: AssetTypeMap,
        scenes: SceneMap,
        viewport: Option<Viewport>,
        paused: bool,
    ) -> Self {
        load_default_assets(render_target, &mut assets);

        let renderer = Renderer::new(&render_target, &mut assets, viewport);
        let renderer2d = Renderer2d::new(&render_target, &mut assets);

        Game {
            renderer,
            renderer2d,
            paused,
            assets,
            scenes,
            scene_initialized: false,
        }
    }

    pub fn update(&mut self, dt: time::Time) {
        if !self.scene_initialized {
            self.scenes
                .get("default")
                .unwrap_or(&(default_scene as SceneFn))(Self::get_scene_builder());
        }
        if self.paused {
            return;
        }
        self.renderer.update(dt);
    }

    pub fn render(&mut self, render_target: &mut RenderTarget) {
        if self.paused {
            return;
        }
        self.renderer.render(render_target, &mut self.assets);
        self.renderer2d.render(render_target);
    }

    pub fn handle_input(&mut self, input: &Input) {
        if self.paused {
            return;
        }
        self.renderer.handle_input(input);
    }

    //TODO: Scenes should be assets loaded with AssetManager struct.
    pub fn set_scene<'s>(&mut self, scene: &'s str) {
        self.scenes[scene](Self::get_scene_builder());
    }

    pub fn get_scene_builder() -> SceneBuilder {
        SceneBuilder::new(todo!())
    }
}
