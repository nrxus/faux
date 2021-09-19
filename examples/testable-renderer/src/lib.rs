// Mocks are available for both for test and when the faux feature is on
#[cfg_attr(any(test, feature = "faux"), faux::create)]
pub struct Renderer {
    // not relevant
    _inner: u8,
}

// Mocks are available for both for test and when the faux feature is on
#[cfg_attr(any(test, feature = "faux"), faux::methods)]
impl Renderer {
    pub fn new() -> Renderer {
        unimplemented!()
    }

    pub fn render(&mut self, _texture: &Texture) -> Result<(), RenderError> {
        unimplemented!()
    }
}

pub struct Texture;

impl Texture {
    pub fn render(&self, renderer: &mut Renderer) -> Result<(), RenderError> {
        renderer.render(self)
    }
}

#[derive(Debug)]
pub struct RenderError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_textures() {
        // mocks work because they are active for the test target
        let mut renderer = Renderer::faux();
        faux::when!(renderer.render).then(|_| Ok(()));

        let subject = Texture {};
        subject.render(&mut renderer).expect("failed to render the texture")
    }
}

// here just to hush warnings
#[cfg_attr(any(test, feature = "faux"), faux::methods)]
impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
