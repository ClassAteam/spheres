use crate::within_pass_renderer::WithinPassRenderer;

/// Manages a collection of renderers where only one is active at a time.
///
/// All renderers are kept initialized and "warm" (ready to render), but only
/// the active renderer is actually rendered via `draw_within_pass()`.
pub struct RendererPool {
    renderers: Vec<Box<dyn WithinPassRenderer>>,
    active_index: usize,
}

impl RendererPool {
    /// Creates a new empty renderer pool.
    pub fn new() -> Self {
        Self {
            renderers: Vec::new(),
            active_index: 0,
        }
    }

    /// Adds a renderer to the pool.
    /// If this is the first renderer, it becomes the active one.
    pub fn add_renderer(&mut self, renderer: Box<dyn WithinPassRenderer>) {
        self.renderers.push(renderer);
    }

    /// Returns a mutable reference to the active renderer.
    ///
    /// # Panics
    /// Panics if the pool is empty.
    pub fn active(&mut self) -> &mut dyn WithinPassRenderer {
        &mut *self.renderers[self.active_index]
    }

    /// Switches the active renderer to the one at the specified index.
    ///
    /// # Returns
    /// `true` if the switch was successful, `false` if the index is out of bounds.
    pub fn switch_active(&mut self, index: usize) -> bool {
        if index < self.renderers.len() {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    /// Returns the index of the currently active renderer.
    pub fn active_index(&self) -> usize {
        self.active_index
    }

    /// Returns the total number of renderers in the pool.
    pub fn len(&self) -> usize {
        self.renderers.len()
    }

    /// Returns `true` if the pool contains no renderers.
    pub fn is_empty(&self) -> bool {
        self.renderers.is_empty()
    }
}

impl Default for RendererPool {
    fn default() -> Self {
        Self::new()
    }
}
