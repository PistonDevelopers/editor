#![deny(missing_docs)]

//! Editor interface.

use std::any::Any;

/// A generic interface for editors, implemented on controllers.
///
/// Provides all information necessary to execute actions,
/// select objects, navigate and update.
/// This makes it possible to write reusable generic actions.
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
    fn delete(&mut self, ty: Type, obj: Object) -> Result<(), ()>;
    /// Updates an object with new values.
    fn update(&mut self, ty: Type, obj: Object, args: &Any) -> Result<(), ()>;
    /// Replaces an object with another.
    fn replace(&mut self, ty: Type, from: Object, to: Object) -> Result<(), ()>;
    /// Get the value of an object.
    fn get<'a>(&'a self, ty: Type, obj: Object) -> Result<&'a Any, ()>;
    /// Get the visible objects of a type.
    fn visible(&self, ty: Type) -> Vec<Object>;
    /// Gets the selected object of a type.
    /// If the editor supports multiple selection,
    /// the selected object is usually the among the multiple-selected ones.
    fn selected(&self, ty: Type) -> Option<Object>;
    /// Gets the multiple selected objects of a type.
    /// The order of the selected objects matter.
    fn multiple_selected(&self, ty: Type) -> Vec<Object>;
    /// Get all objects of a type.
    fn all(&self, ty: Type) -> Vec<Object>;
    /// Navigate to an object such that it becomes visible.
    fn navigate_to(&mut self, ty: Type, obj: Object) -> Result<(), ()>;
}

/// The type of an object.
/// This does not have be unique for Rust types.
/// Dynamically typed objects should use same id.
#[derive(Clone, Copy, Debug)]
pub struct Type(pub &'static str);
/// The object id.
#[derive(Clone, Copy, Debug)]
pub struct Object(pub usize);

/// A helper function for `Editor::delete` implementation.
pub fn delete<T>(items: &mut Vec<T>, obj: Object) -> Result<Option<Object>, ()> {
    if items.len() == 0 { return Err(()); }

    let upd_obj = if obj.0 == items.len() - 1 {
        // The deleted object was last, no update needed.
        None
    } else {
        // Update references to the last object,
        // which now takes the place of the deleted object.
        Some(Object(items.len() - 1))
    };
    items.swap_remove(obj.0);
    Ok(upd_obj)
}

/// A helper function for `Editor::update` implementation.
pub fn update<T: Any + Clone>(items: &mut Vec<T>, obj: Object, args: &Any)
-> Result<(), ()> {
    match args.downcast_ref::<T>() {
        None => { return Err(()); }
        Some(val) => {
            items[obj.0] = val.clone();
            Ok(())
        }
    }
}

/// A helper function for `Editor::all` implementation.
pub fn all<T>(items: &Vec<T>) -> Vec<Object> {
    (0..items.len()).map(|i| Object(i)).collect()
}

/// A helper function for `Editor::get` implementation.
pub fn get<T: Any>(items: &Vec<T>, obj: Object) -> Result<&Any, ()> {
    Ok(try!(items.get(obj.0).ok_or(())))
}
