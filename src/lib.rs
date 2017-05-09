//! A tree view implementation for [cursive](https://crates.io/crates/cursive).
#![deny(
    missing_docs,
    trivial_casts, trivial_numeric_casts,
    unsafe_code,
    unused_import_braces, unused_qualifications
)]

// Crate Dependencies ---------------------------------------------------------
extern crate cursive;


// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Display;


// External Dependencies ------------------------------------------------------
use cursive::With;
use cursive::vec::Vec2;
use cursive::view::{ScrollBase, View};
use cursive::theme::ColorStyle;
use cursive::{Cursive, Printer};
use cursive::direction::Direction;
use cursive::event::{Callback, Event, EventResult, Key};


// Internal Dependencies ------------------------------------------------------
mod tree_list;
use tree_list::TreeList;
pub use tree_list::Placement;


/// View to select an item among a tree.
///
/// # Examples
///
/// ```
/// # extern crate cursive;
/// # extern crate cursive_tree_view;
/// # use cursive_tree_view::{TreeView, Placement};
/// # fn main() {
/// let mut tree = TreeView::new();
///
/// tree.insert_item("root".to_string(), Placement::Child, 0);
///
/// tree.insert_item("1".to_string(), Placement::Child, 0);
/// tree.insert_item("2".to_string(), Placement::Child, 1);
/// tree.insert_item("3".to_string(), Placement::Child, 2);
/// # }
/// ```
pub struct TreeView<T: Display> {
    enabled: bool,
    on_submit: Option<Rc<Fn(&mut Cursive, usize)>>,
    on_select: Option<Rc<Fn(&mut Cursive, usize)>>,
    on_collapse: Option<Rc<Fn(&mut Cursive, usize, bool)>>,

    scrollbase: ScrollBase,
    last_size: Vec2,
    focus: usize,
    list: TreeList<T>
}

impl<T: Display> TreeView<T> {

    /// Creates a new, empty `TreeView`.
    pub fn new() -> Self {
        Self {
            enabled: true,
            on_submit: None,
            on_select: None,
            on_collapse: None,

            scrollbase: ScrollBase::new(),
            last_size: (0, 0).into(),
            focus: 0,
            list: TreeList::new()
        }
    }

    /// Disables this view.
    ///
    /// A disabled view cannot be selected.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Re-enables this view.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Enable or disable this view.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns `true` if this view is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets a callback to be used when `<Enter>` is pressed while an item
    /// is selected.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.set_on_submit(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// ```
    pub fn set_on_submit<F>(&mut self, cb: F)
        where F: Fn(&mut Cursive, usize) + 'static
    {
        self.on_submit = Some(Rc::new(move |s, row| cb(s, row)));
    }

    /// Sets a callback to be used when `<Enter>` is pressed while an item
    /// is selected.
    ///
    /// Chainable variant.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.on_submit(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// ```
    pub fn on_submit<F>(self, cb: F) -> Self
        where F: Fn(&mut Cursive, usize) + 'static
    {
        self.with(|t| t.set_on_submit(cb))
    }

    /// Sets a callback to be used when an item is selected.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.set_on_select(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// ```
    pub fn set_on_select<F>(&mut self, cb: F)
        where F: Fn(&mut Cursive, usize) + 'static
    {
        self.on_select = Some(Rc::new(move |s, row| cb(s, row)));
    }

    /// Sets a callback to be used when an item is selected.
    ///
    /// Chainable variant.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.on_select(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// ```
    pub fn on_select<F>(self, cb: F) -> Self
        where F: Fn(&mut Cursive, usize) + 'static
    {
        self.with(|t| t.set_on_select(cb))
    }

    /// Sets a callback to be used when an item has its children collapsed or expanded.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.set_on_collapse(|siv: &mut Cursive, row: usize, collapsed: bool| {
    ///
    /// });
    /// ```
    pub fn set_on_collapse<F>(&mut self, cb: F)
        where F: Fn(&mut Cursive, usize, bool) + 'static
    {
        self.on_collapse = Some(Rc::new(move |s, row, collapsed| cb(s, row, collapsed)));
    }

    /// Sets a callback to be used when an item has its children collapsed or expanded.
    ///
    /// Chainable variant.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.on_collapse(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// ```
    pub fn on_collapse<F>(self, cb: F) -> Self
        where F: Fn(&mut Cursive, usize, bool) + 'static
    {
        self.with(|t| t.set_on_collapse(cb))
    }

    /// Removes all items from this view.
    pub fn clear(&mut self) {
        self.list.clear();
        self.focus = 0;
    }

    /// Removes all items from this view, returning them.
    pub fn take_items(&mut self) -> Vec<T> {
        let items = self.list.take_items();
        self.focus = 0;
        items
    }

    /// Returns the number of items in this table.
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Returns `true` if this table has no item.
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Returns the index of the currently selected table row.
    pub fn row(&self) -> Option<usize> {
        if self.is_empty() {
            None

        } else {
            Some(self.focus)
        }
    }

    /// Selects the row at the specified index.
    pub fn set_selected_row(&mut self, row_index: usize) {
        self.focus = row_index;
        self.scrollbase.scroll_to(row_index);
    }

    /// Selects the row at the specified index.
    ///
    /// Chainable variant.
    pub fn selected_row(self, row_index: usize) -> Self {
        self.with(|t| t.set_selected_row(row_index))
    }

    /// Returns a immmutable reference to the item at the given row.
    pub fn borrow_item(&mut self, row_index: usize) -> Option<&T> {
        let index = self.list.visual_index(row_index);
        self.list.get(index)
    }

    /// Returns a mutable reference to the item at the given row.
    pub fn borrow_item_mut(&mut self, row_index: usize) -> Option<&mut T> {
        let index = self.list.visual_index(row_index);
        self.list.get_mut(index)
    }

    /// Inserts a new `item` at the given `row` with the specified
    /// [`Placement`](enum.Placement.html), returning the row index of the item
    /// occupies after its insertion.
    pub fn insert_item(&mut self, item: T, placement: Placement, row: usize) -> usize {
        let index = self.list.visual_index(row);
        self.list.insert(placement, index, item)
    }

    /// Removes the item at the given `row` along with all of its children.
    ///
    /// The returned vector contains the removed items in top to bottom order.
    pub fn remove_item(&mut self, row: usize) -> Option<Vec<T>> {
        let index = self.list.visual_index(row);
        let removed = self.list.remove_with_children(index);
        self.focus = cmp::min(self.focus, self.list.height() - 1);
        removed
    }

    /// Extracts the item at the given `row` from the tree.
    ///
    /// All of the items children will be moved up one level within the tree.
    pub fn extract_item(&mut self, row: usize) -> Option<T> {
        let index = self.list.visual_index(row);
        let removed = self.list.remove(index);
        self.focus = cmp::min(self.focus, self.list.height() - 1);
        removed
    }

    /// Collapses the children of the given `row`.
    pub fn collapse_item(&mut self, row: usize) {
        let index = self.list.visual_index(row);
        self.list.set_collapsed(index, true);
    }

    /// Expands the children of the given `row`.
    pub fn expand_item(&mut self, row: usize) {
        let index = self.list.visual_index(row);
        self.list.set_collapsed(index, false);
    }

    /// Collapses or expands the children of the given `row`.
    pub fn set_collapsed(&mut self, row: usize, collapsed: bool) {
        let index = self.list.visual_index(row);
        self.list.set_collapsed(index, collapsed);
    }

    /// Collapses or expands the children of the given `row`.
    ///
    /// Chained variant.
    pub fn collapsed(self, row: usize, collapsed: bool) -> Self {
        self.with(|t| t.set_collapsed(row, collapsed))
    }

}

impl<T: Display> TreeView<T> {

    fn focus_up(&mut self, n: usize) {
        self.focus -= cmp::min(self.focus, n);
    }

    fn focus_down(&mut self, n: usize) {
        self.focus = cmp::min(self.focus + n, self.list.height() - 1);
    }

}

impl<T: Display> View for TreeView<T> {

    fn draw(&self, printer: &Printer) {

        let items = self.list.items();
        let list_index = Rc::new(RefCell::new(self.scrollbase.start_line));

        self.scrollbase.draw(printer, |printer, i| {

            let mut index = list_index.borrow_mut();
            let item = &items[*index];

            if item.collapsed {
                *index += item.children + 1;

            } else {
                *index += 1;
            };

            let color = if i == self.focus {
                if self.enabled && printer.focused {
                    ColorStyle::Highlight

                } else {
                    ColorStyle::HighlightInactive
                }

            } else {
                ColorStyle::Primary
            };

            if item.children > 0 {
                if item.collapsed {
                    printer.print((item.level * 2, 0), "▸");

                } else {
                    printer.print((item.level * 2, 0), "▾");
                }

            } else {
                printer.print((item.level * 2, 0), "◦");
            }

            printer.with_color(color, |printer| {
                printer.print(
                    (item.level * 2 + 2, 0),
                    format!("{}", item.value).as_str()
                );
            });

        });

    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {

        let width: usize = self.list.items().iter().map(|item| {
            item.level * 2 + format!("{}", item.value).len() + 2

        }).max().unwrap_or(0);

        let h = self.list.height();
        let w = if req.y < h {
            width + 2

        } else {
            width
        };

        (w, h).into()

    }

    fn layout(&mut self, size: Vec2) {
        let height = self.list.height();
        self.scrollbase.set_heights(size.y, height);
        self.scrollbase.scroll_to(self.focus);
        self.last_size = size;
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        self.enabled && !self.is_empty()
    }

    fn on_event(&mut self, event: Event) -> EventResult {

        if !self.enabled {
            return EventResult::Ignored;
        }

        let last_focus = self.focus;
        match event {
            Event::Key(Key::Up) if self.focus > 0 => {
                self.focus_up(1);
            },
            Event::Key(Key::Down) if self.focus + 1 < self.list.height() => {
                self.focus_down(1);
            },
            Event::Key(Key::PageUp) => {
                self.focus_up(10);
            },
            Event::Key(Key::PageDown) => {
                self.focus_down(10);
            }
            Event::Key(Key::Home) => {
                self.focus = 0;
            },
            Event::Key(Key::End) => {
                self.focus = self.list.height() - 1;
            },
            Event::Key(Key::Enter) => if !self.is_empty() {

                let row = self.focus;
                let index = self.list.visual_index(row);
                let collapsed = self.list.get_collapsed(index);
                let children = self.list.get_children(index);

                if children > 0 {

                    self.list.set_collapsed(index, !collapsed);

                    if self.on_collapse.is_some() {
                        let cb = self.on_collapse.clone().unwrap();
                        return EventResult::Consumed(Some(Callback::from_fn(move |s| {
                            cb(s, row, !collapsed)
                        })));
                    }

                } else if self.on_submit.is_some() {
                    let cb = self.on_submit.clone().unwrap();
                    return EventResult::Consumed(Some(Callback::from_fn(move |s| {
                        cb(s, row)
                    })));
                }
            },
            _ => return EventResult::Ignored
        }

        let focus = self.focus;
        self.scrollbase.scroll_to(focus);

        if !self.is_empty() && last_focus != focus {
            let row = self.focus;
            EventResult::Consumed(self.on_select.clone().map(|cb| {
                Callback::from_fn(move |s| cb(s, row))
            }))

        } else {
            EventResult::Ignored
        }

    }

}

