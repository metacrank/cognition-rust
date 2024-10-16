use crate::pool::Pool;
use core::cmp;
use core::cmp::Ordering;
use core::fmt::Display;

pub struct Tree<K:Ord+Copy+Display,D> {
  pub root: Option<Box<Node<K,D>>>
}

pub struct Node<K:Ord+Display,D> {
  key: K,
  data: Option<Vec<D>>,
  height: i32,
  node_left: Option<Box<Node<K,D>>>,
  node_right: Option<Box<Node<K,D>>>,
}

impl<K:Ord+Copy+Display,D> Tree<K,D> {
  pub fn new() -> Self {
    Self{ root: None }
  }

  pub fn print(&self) {
    if let Some(r) = &self.root { r.print(); println!("") };
  }

  pub fn insert(&mut self, key: K, data: D, new: fn(&mut Pool) -> Vec<D>, p: &mut Pool) {
    match self.root.take() {
      Some(r) => self.root = Some(Node::insert(key, data, r, new, p)),
      None => {
        let mut vec = new(p);
        vec.push(data);
        self.root = Some(Box::new(Node::new(key, vec)))
      },
    }
  }

  pub fn remove_at_least(&mut self, key: K, free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    if self.root.is_none() { return None };
    Node::remove_at_least(key, &mut self.root, free, p)
  }
  pub fn remove_least_at_least(&mut self, key: K, free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    if self.root.is_none() { return None };
    Node::remove_least_at_least(key, &mut self.root, free, p)
  }

  // Currently does nothing
  pub fn gc(&mut self, free: fn(&mut Pool, Vec<D>), p: &mut Pool) {
    if let Some(r) = self.root.take() { self.root = Node::delete_lowest_nodes(r, free, p) }
  }
}

impl<K:Ord+Display,D> Node<K,D> {
  pub fn new(key: K, data: Vec<D>) -> Self {
    Self{key, data: Some(data), height: 1, node_left: None, node_right: None}
  }

  fn print(&self) {
    print!("(");
    if let Some(l) = &self.node_left {
      l.print();
      print!(" ");
    }
    match &self.data {
      Some(data) => print!("{}: {}", self.key, data.len()),
      None       => print!("{}: 0", self.key),
    }
    if let Some(r) = &self.node_right {
      print!(" ");
      r.print();
    }
    print!(")");
  }

  fn height(node: &Option<Box<Node<K,D>>>) -> i32 {
    node.as_ref().map_or(0, |n| n.height)
  }

  fn update_height(root: &mut Node<K,D>){
    root.height = cmp::max( Self::height(&root.node_left), Self::height(&root.node_right) ) + 1;
  }

  fn rotate_right(mut root: Box<Node<K,D>>) -> Box<Node<K,D>>{
    let mut newroot = root.node_left.take().expect("Tree error: invalid rotate");
    root.node_left = newroot.node_right.take();
    Self::update_height(&mut root);
    newroot.node_right = Some(root);
    newroot
  }

  fn rotate_left(mut root: Box<Node<K,D>>) -> Box<Node<K,D>>{
    let mut newroot = root.node_right.take().expect("Tree error: invalid rotate");
    root.node_right = newroot.node_left.take();
    Self::update_height(&mut root);
    newroot.node_left = Some(root);
    newroot
  }

  // If the left node is too high
  fn rotate_left_node(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
    let left = root.node_left.take().expect("Tree error: invalid rotate");
    if Self::height(&left.node_left) < Self::height(&left.node_right) {
      let rotated = Self::rotate_left(left);
      root.node_left = Some(rotated);
      Self::update_height(&mut root);
    } else {
      root.node_left = Some(left);
    }
    Self::rotate_right(root)
  }

  // If the right node is too high
  fn rotate_right_node(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
    let right = root.node_right.take().expect("Tree error: invalid rotate");
    if Self::height(&right.node_left) > Self::height(&right.node_right) {
      let rotated = Self::rotate_right(right);
      root.node_right = Some(rotated);
      Self::update_height(&mut root);
    } else {
      root.node_right = Some(right);
    }
    Self::rotate_left(root)
  }

  fn left_minus_right(root: &Box<Node<K,D>>) -> i32 {
    let left = Self::height(&root.node_left);
    let right = Self::height(&root.node_right);
    left - right
  }

  fn rotate_correctly(root: Box<Node<K,D>>) -> Box<Node<K,D>> {
    let diff = Self::left_minus_right(&root);
    if -1 <= diff && diff <= 1 { return root };
    match diff {
      2 => Self::rotate_left_node(root),
      -2 => Self::rotate_right_node(root),
      _ => unreachable!(),
    }
  }

  fn insert_into_node(key: K, data: D, node: Option<Box<Node<K,D>>>, new: fn(&mut Pool) -> Vec<D>, p: &mut Pool) -> Option<Box<Node<K,D>>> {
    Some(match node {
      Some(n) => Self::insert(key, data, n, new, p),
      None => {
        let mut vec = new(p);
        vec.push(data);
        Box::new(Node::new(key, vec))
      },
    })
  }

  fn insert(key: K, data: D, mut root: Box<Node<K,D>>, new: fn(&mut Pool) -> Vec<D>, p: &mut Pool) -> Box<Node<K,D>> {
    match root.key.cmp(&key) {
      Ordering::Equal => {
        root.data.as_mut().expect("Tree error: invalid node").push(data);
        return root;
      },
      Ordering::Less =>    root.node_right = Self::insert_into_node(key, data, root.node_right.take(), new, p),
      Ordering::Greater => root.node_left  = Self::insert_into_node(key, data, root.node_left.take(), new, p),
    }
    Self::update_height(&mut *root);
    Self::rotate_correctly(root)
  }

  fn updated_node(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
    Self::update_height(&mut root);
    Self::rotate_correctly(root)
  }

  fn drop_min_from_left(mut root: Box<Node<K,D>>, left: Box<Node<K,D>>) -> (Option<Box<Node<K,D>>>, Box<Node<K,D>>) {
    let (new_left, min) = Self::drop_min(left);
    root.node_left = new_left;
    (Some(Self::updated_node(root)), min)
  }

  fn drop_min(mut root: Box<Node<K,D>>) -> (Option<Box<Node<K,D>>>, Box<Node<K,D>>) {
    match root.node_left.take() {
      Some(left) => Self::drop_min_from_left(root, left),
      None => (root.node_right.take(), root),
    }
  }

  fn combine_branches(l: Box<Node<K,D>>, r: Box<Node<K,D>>) -> Box<Node<K,D>> {
    let (trunc_right, min) = Self::drop_min(r);
    let mut newroot = min;
    newroot.node_left = Some(l);
    newroot.node_right = trunc_right;
    Self::updated_node(newroot)
  }

  fn delete(mut root: Box<Node<K,D>>) -> Option<Box<Node<K,D>>> {
    match (root.node_left.take(), root.node_right.take()) {
      (None, None) => None,
      (Some(l), None) => Some(l),
      (None, Some(r)) => Some(r),
      (Some(l), Some(r)) => Some(Self::combine_branches(l, r)),
    }
  }

  fn pop_from_root(root: &mut Option<Box<Node<K,D>>>, free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    let Some(rootnode) = root else { return None };
    let vec = rootnode.data.as_mut().expect("Tree error: invalid node");
    let retval = vec.pop();
    if vec.len() == 0 {
      free(p, rootnode.data.take().expect("Tree error: invalid node"));
      *root = Self::delete(root.take().unwrap());
    }
    retval
  }

  fn remove_at_least(key: K, root: &mut Option<Box<Node<K,D>>>, free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    let Some(rootnode) = root else { return None };
    if rootnode.key.cmp(&key).is_ge() {
      Self::pop_from_root(root, free, p)
    } else {
      let retval = Self::remove_at_least(key, &mut rootnode.node_right, free, p);
      *root = Some(Self::updated_node(root.take().expect("Unreachable in Node::remove_at_least()")));
      retval
    }
  }

  fn remove_least_at_least(key: K, root: &mut Option<Box<Node<K,D>>>, free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    let Some(rootnode) = root else { return None };
    match rootnode.key.cmp(&key) {
      Ordering::Equal => {
        Self::pop_from_root(root, free, p)
      },
      Ordering::Less => {
        let retval = Self::remove_at_least(key, &mut rootnode.node_right, free, p);
        *root = Some(Self::updated_node(root.take().expect("Unreachable in Node::remove_at_least()")));
        retval
      },
      Ordering::Greater => {
        if rootnode.node_left.is_none() { return Self::pop_from_root(root, free ,p) };
        let retval = Self::remove_at_least(key, &mut rootnode.node_left, free, p);
        if retval.is_none() { return Self::pop_from_root(root, free, p) };
        *root = Some(Self::updated_node(root.take().expect("Unreachable in Node::remove_at_least()")));
        retval
      },
    }
  }

  fn delete_lowest_nodes(root: Box<Node<K,D>>, _free: fn(&mut Pool, Vec<D>), _p: &mut Pool) -> Option<Box<Node<K,D>>> {
    Some(root)
  }
}
