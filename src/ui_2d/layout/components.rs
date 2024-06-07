use bevy::prelude::*;
use bevy_cobweb::prelude::*;


//-------------------------------------------------------------------------------------------------------------------

/// Component for entities that own UI trees.
///
/// Note that root entities are *not* UI nodes. Typically they are cameras, textures, or entities in world space.
/// UI node trees sit on top of UI roots.
#[derive(Component, Debug, Copy, Clone)]
pub struct Ui2DRoot
{
    /// Defines the base z-offset applied between the entity and its node children.
    ///
    /// For example, this is set to a negative value for cameras nodes attached to cameras so UI elements will be in-view of the
    /// camera (see [`DEFAULT_CAMERA_Z_OFFSET`](crate::DEFAULT_CAMERA_Z_OFFSET)).
    pub base_z_offset: f32,
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for 2d UI nodes.
#[derive(Component, Default, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct CobwebNode;

//-------------------------------------------------------------------------------------------------------------------

/// Component with the [`SizeRef`] of the base ancestor of a node.
///
/// The base ancestor of a node is the last ancestor before you reach the [`Ui2DRoot`] (i.e. it's the first actual
/// node in the tree that sits on a [`Ui2DRoot`]).
///
/// This is updated in [`LayoutSetCompute`].
#[derive(Component, Debug, PartialEq, Copy, Clone)]
pub struct BaseSizeRef
{
    pub base: Entity,
    pub sizeref: SizeRef,
}

impl Default for BaseSizeRef
{
    fn default() -> Self
    {
        Self{ base: Entity::PLACEHOLDER, sizeref: SizeRef::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component with the size reference for computing the layout of a node.
///
/// Typically this equals the parent's [`NodeSize`], or is derived from a [`UiCamera2D`].
///
/// This is updated in [`LayoutSetCompute`].
#[derive(Component, Default, Debug, PartialEq, Copy, Clone, Deref, DerefMut)]
pub struct SizeRef(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

/// Reactive component that records the 2D dimensions of a node as a rectangle on the plane of the node's parent.
#[derive(ReactComponent, Default, Debug, PartialEq, Copy, Clone, Deref, DerefMut)]
pub struct NodeSize(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------
