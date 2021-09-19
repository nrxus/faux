use testable_renderer::{RenderError, Renderer, Texture};

struct World {
    player: Texture,
    enemy: Texture,
}

impl World {
    pub fn new() -> Self {
        World {
            player: Texture {},
            enemy: Texture {},
        }
    }

    pub fn render(&self, renderer: &mut Renderer) -> Result<(), RenderError> {
        self.player.render(renderer)?;
        self.enemy.render(renderer)?;

        Ok(())
    }
}

fn main() {
    let world = World::new();
    let mut renderer = Renderer::new();
    world.render(&mut renderer).expect("oops couldn't render");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_world() {
        // the test target enables the faux feature on `testable-renderer`
        // thus allowing us to use the mocks of the *external* crate
        let mut renderer = Renderer::faux();
        faux::when!(renderer.render).then(|_| Ok(()));

        let world = World::new();
        world.render(&mut renderer).expect("failed to render the world")
    }
}
