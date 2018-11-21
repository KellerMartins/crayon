//! Named bucket of draw calls with the wrapping of rendering operations to a render
//! target, clearing, MSAA resolving and so on.

use math::prelude::{Color, Vector2};

use video::assets::texture::RenderTextureHandle;
use video::errors::{Error, Result};
use video::MAX_FRAMEBUFFER_ATTACHMENTS;

/// The setup data of `Surface` which wraps common rendering operations to a render-target.
/// Likes clearing, MSAA resolves, etc.. The `RenderTarget` is the window framebuffer as
/// default, but you can specify `RenderTarget` with `SurfaceParams::set_attachments`
/// manually also.
#[derive(Debug, Copy, Clone)]
pub struct SurfaceParams {
    pub(crate) colors: [Option<RenderTextureHandle>; MAX_FRAMEBUFFER_ATTACHMENTS],
    pub(crate) depth_stencil: Option<RenderTextureHandle>,
    pub(crate) clear_color: Option<Color<f32>>,
    pub(crate) clear_depth: Option<f32>,
    pub(crate) clear_stencil: Option<i32>,
}

impl Default for SurfaceParams {
    fn default() -> Self {
        SurfaceParams {
            colors: [None; MAX_FRAMEBUFFER_ATTACHMENTS],
            depth_stencil: None,
            clear_color: Some(Color::black()),
            clear_depth: Some(1.0),
            clear_stencil: None,
        }
    }
}

impl_handle!(SurfaceHandle);

impl SurfaceParams {
    /// Sets the attachments of internal frame-buffer. It consists of multiple color attachments
    /// and a optional `Depth/DepthStencil` buffer attachment.
    ///
    /// If none attachment is assigned, the default framebuffer generated by the system will be
    /// used.
    pub fn set_attachments<T1>(
        &mut self,
        colors: &[RenderTextureHandle],
        depth_stencil: T1,
    ) -> Result<()>
    where
        T1: Into<Option<RenderTextureHandle>>,
    {
        if colors.len() >= MAX_FRAMEBUFFER_ATTACHMENTS {
            return Err(Error::SurfaceInvalid("Too many color attachments.".into()));
        }

        for (i, v) in self.colors.iter_mut().enumerate() {
            if i < colors.len() {
                *v = Some(colors[i]);
            } else {
                *v = None;
            }
        }

        self.depth_stencil = depth_stencil.into();
        Ok(())
    }

    /// Sets the clear flags for this surface.A
    #[inline]
    pub fn set_clear<C, D, S>(&mut self, color: C, depth: D, stentil: S)
    where
        C: Into<Option<Color<f32>>>,
        D: Into<Option<f32>>,
        S: Into<Option<i32>>,
    {
        self.clear_color = color.into();
        self.clear_depth = depth.into();
        self.clear_stencil = stentil.into();
    }
}

/// Defines a rectangle, called the scissor box, in window coordinates. The test is
/// initially disabled. While the test is enabled, only pixels that lie within the
/// scissor box can be modified by drawing commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceScissor {
    Enable {
        /// Specify the lower left corner of the scissor box, in pixels. Initially (0, 0).
        position: Vector2<i32>,
        /// Specify the width and height of the scissor box. When a surface is first attached to
        /// a window, width and height are set to the dimensions of that window.
        size: Vector2<u32>,
    },
    Disable,
}

/// Sets the viewport of surface. This specifies the affine transformation of (x, y),
/// in window coordinates to normalized window coordinates.
///
/// NDC(normalized device coordinates) to normalized window coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceViewport {
    /// Specify the lower left corner of the viewport rectangle, in pixels. Initially (0, 0).
    pub position: Vector2<i32>,
    /// Specify the width and height of the viewport. When a surface is first attached to
    /// a window, width and height are set to the dimensions of that window.
    pub size: Vector2<u32>,
}