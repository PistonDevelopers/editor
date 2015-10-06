#![deny(missing_docs)]

//! Editor interface.

use std::any::Any;
use std::sync::Arc;

/// A generic interface for editors, implemented on controllers.
///
/// Provides all information necessary to execute actions,
/// select objects, navigate and update.
/// This makes it possible to write reusable generic actions.
///
/// History is handled externally, using `Box<Any>` for changes.
/// The editor should not store any history internally,
/// unless it is garbage collected from within the `Any` data sent outside.
///
/// All changes must be reversible from the information that is sent outside.
/// You can use `Arc` to check for uniqueness for resources with a handle.
/// If the internal `Arc` is unique, it means the information can safely be
/// removed. This must happen internally.
///
/// A history buffer must keep handles alive since they can not be recreated.
/// If this is a problem, then the editor must map to a recreatable resource.
///
/// The controller should keep same selection state across multiple views.
///
/// References should not be handled internally.
/// This is done through algorithms using the reference information.
/// For example, before deleting an object, checks that all affected
/// references are cascading, such there are no loose references after deletion.
/// Cascading references deletes the objects that the reference points from.
///
/// Methods that returns `Result` can trigger a rollback in actions.
/// This is to prevent logical errors from affecting data.
/// Concurrent actions are not permitted at the same time.
///
/// View information must be stored internally in the editor.
/// If the editor state depends on the view state, then it should not be
/// updated before `refresh_views` is called.
pub trait Editor {
    /// Gets the current cursor position in 2D.
    fn cursor_2d(&self) -> Option<[f64; 2]>;
    /// Gets the current cursor position in 3D world coordinates.
    fn cursor_3d(&self) -> Option<[f64; 3]>;
    /// Try to hit objects at 2D position.
    fn hit_2d(&self, pos: [f64; 2]) -> Vec<(Type, Object)>;
    /// Try to hit objects at 3D position.
    fn hit_3d(&self, pos: [f64; 3]) -> Vec<(Type, Object)>;
    /// Select a single object.
    fn select(&mut self, ty: Type, obj: Object) -> Result<(), ()>;
    /// Select multiple objects.
    /// Adds to the current selection.
    fn select_multiple(&mut self, ty: Type, objs: &[Object]) -> Result<(), ()>;
    /// Deselect multiple objects.
    /// Removes from the current selection.
    fn deselect_multiple(&mut self, ty: Type, objs: &[Object]) -> Result<(), ()>;
    /// Deselect everything of a type.
    fn select_none(&mut self, ty: Type) -> Result<(), ()>;
    /// Inserts a new object.
    fn insert(&mut self, ty: Type, args: &Any) -> Result<Object, ()>;
    /// Returns an object which references must be updated when
    /// using swap-remove by replacing object with last one in same table.
    fn delete(&mut self, ty: Type, obj: Object) -> Result<Option<Object>, ()>;
    /// Updates an object with new values.
    fn update(&mut self, ty: Type, args: &Any) -> Result<(), ()>;
    /// Replaces an object with another.
    /// Keeps references pointing to the old object, but deletes
    /// references pointing from the old object.
    fn replace(&mut self, ty: Type, from: Object, to: Object)
    -> Result<Option<Object>, ()>;
    /// Get the field of an object.
    fn get<'a>(&'a self, ty: Type, obj: Object) -> Result<&'a Any, ()>;
    /// Get references pointing to an object.
    fn references_to(&self, ty: Type, obj: Object) -> Vec<Reference>;
    /// Get references pointing from an object to other objects.
    fn references_from(&self, ty: Type, obj: Object) -> Vec<Reference>;
    /// Get the visible objects of a type.
    fn visible(&self, ty: Type) -> Vec<Object>;
    /// Gets the selected object of a type.
    /// If the editor supports multiple selection,
    /// the selected object is usually the last in the selected list.
    fn selected(&self, ty: Type) -> Option<Object>;
    /// Gets the multiple selected objects of a type.
    /// The order of the selected objects matter.
    fn multiple_selected(&self, ty: Type) -> Vec<Object>;
    /// Get all objects of a type.
    fn all(&self, ty: Type) -> Vec<Object>;
    /// Navigate to an object such that it becomes visible.
    fn navigate_to(&mut self, ty: Type, obj: Object) -> Result<(), ()>;
    /// Gets the types in the editor.
    fn types(&self) -> Vec<Type>;
    /// Get the fields of an object.
    /// This requires an object because it can be dynamically typed.
    /// Fields of statically types are known at compile time.
    fn fields_of(&self, ty: Type, obj: Object) -> Vec<Field>;
    /// Updates a field. This is used by property widgets.
    fn update_field(&mut self, ty: Type, obj: Object, field: Field, val: &Any)
    -> Result<(), ()>;
    /// Refreshes the views.
    /// This is called at the end of each action to update cached data.
    fn refresh_views(&mut self);
}

/// The type of an object.
/// This does not have be unique for Rust types.
/// Dynamically typed objects should use same id.
#[derive(Clone, Copy)]
pub struct Type(pub &'static str);
/// The object id.
#[derive(Clone, Copy)]
pub struct Object(pub usize);

/// Stores information about a reference.
pub struct Reference {
    /// The type of the from object.
    pub from_ty: Type,
    /// The id of the from object.
    pub from_obj: Object,
    /// The type of the to object.
    pub to_type: Type,
    /// The id of the to object.
    pub to_obj: Object,
    /// Whether to delete objects using this reference.
    /// When `false`, deletion will be cancelled with an error.
    pub cascade: bool,
}

/// Field information.
pub struct Field {
    /// The name of field.
    pub name: Arc<String>,
    /// The type of the field.
    pub ty: Type,
    /// The index within array, 0 for normal fields.
    pub index: usize,
    /// 0 for a normal named field, length for array.
    pub array: usize,
}
