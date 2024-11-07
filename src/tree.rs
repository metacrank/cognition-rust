use crate::pool::Pool;
use core::cmp;
use core::cmp::Ordering;
use core::fmt::Display;

pub struct Tree<K:Ord+Copy+Display,D> {
  pub root: Option<Box<Node<K,D>>>
}

pub struct Node<K:Ord+Display,D> {
  pub key: K,
  pub data: Option<Vec<D>>,
  pub height: i32,
  pub node_left: Option<Box<Node<K,D>>>,
  pub node_right: Option<Box<Node<K,D>>>,
}

impl<K:Ord+Copy+Display,D> Tree<K,D> {
  pub fn new() -> Self {
    Self{ root: None }
  }

  pub fn print(&self) {
    if let Some(r) = &self.root { r.print() };
  }

  pub fn count(&self) -> usize {
    Node::count(&self.root)
  }

  pub fn insert(&mut self, key: K, data: D, node_new: fn(&mut Pool, K, Vec<D>) -> Box<Node<K,D>>, new: fn(&mut Pool) -> Vec<D>, p: &mut Pool) {
    match self.root.take() {
      Some(r) => self.root = Some(Node::insert(key, data, r, node_new, new, p)),
      None => {
        let mut vec = new(p);
        vec.push(data);
        self.root = Some(node_new(p, key, vec))
      },
    }
  }

  pub fn remove_at_least(&mut self, key: K, drop_node: fn(&mut Pool, Box<Node<K,D>>), free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    if self.root.is_none() { return None };
    Node::remove_at_least(key, &mut self.root, drop_node, free, p)
  }
  pub fn remove_least_at_least(&mut self, key: K, drop_node: fn(&mut Pool, Box<Node<K,D>>), free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    if self.root.is_none() { return None };
    Node::remove_least_at_least(key, &mut self.root, drop_node, free, p)
  }

  pub fn gc(&mut self, free: fn(&mut Pool, Vec<D>), p: &mut Pool) {
    if let Some(root) = self.root.take() {
      self.root = Node::gc(root, free, p);
    }
  }
}

impl<K:Ord+Display,D> Node<K,D> {
  pub fn new(key: K, data: Vec<D>) -> Self {
    Self{ key, data: Some(data), height: 1, node_left: None, node_right: None }
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

  fn count(node: &Option<Box<Self>>) -> usize {
    let Some(n) = node else { return 0 };
    let contrib = if let Some(ref d) = n.data { d.len() } else { 0 };
    Self::count(&n.node_left) + contrib + Self::count(&n.node_right)
  }

  fn height(node: &Option<Box<Self>>) -> i32 {
    node.as_ref().map_or(0, |n| n.height)
  }

  fn update_height(&mut self){
    self.height = cmp::max( Self::height(&self.node_left), Self::height(&self.node_right) ) + 1;
  }

  fn rotate_right(mut root: Box<Self>) -> Box<Self>{
    let mut newroot = root.node_left.take().expect("Tree error: invalid rotate");
    root.node_left = newroot.node_right.take();
    Self::update_height(&mut root);
    newroot.node_right = Some(root);
    Self::update_height(&mut newroot);
    newroot
  }

  fn rotate_left(mut root: Box<Self>) -> Box<Self>{
    let mut newroot = root.node_right.take().expect("Tree error: invalid rotate");
    root.node_right = newroot.node_left.take();
    Self::update_height(&mut root);
    newroot.node_left = Some(root);
    Self::update_height(&mut newroot);
    newroot
  }

  // If the left node is too high
  fn rotate_left_node(mut root: Box<Self>) -> Box<Self> {
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
  fn rotate_right_node(mut root: Box<Self>) -> Box<Self> {
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

  fn left_minus_right(&self) -> i32 {
    let left = Self::height(&self.node_left);
    let right = Self::height(&self.node_right);
    left - right
  }

  fn rotate_correctly(root: Box<Self>) -> Box<Self> {
    let diff = Self::left_minus_right(&root);
    if -1 <= diff && diff <= 1 { return root };
    match diff {
      2 => Self::rotate_left_node(root),
      -2 => Self::rotate_right_node(root),
      _ => unreachable!()
    }
  }

  fn insert_into_node(key: K, data: D, node: Option<Box<Self>>, node_new: fn(&mut Pool, K, Vec<D>) -> Box<Self>,
                      new: fn(&mut Pool) -> Vec<D>, p: &mut Pool) -> Option<Box<Self>> {
    Some(match node {
      Some(n) => Self::insert(key, data, n, node_new, new, p),
      None => {
        let mut vec = new(p);
        vec.push(data);
        node_new(p, key, vec)
      },
    })
  }

  fn insert(key: K, data: D, mut root: Box<Self>, node_new: fn(&mut Pool, K, Vec<D>) -> Box<Self>,
            new: fn(&mut Pool) -> Vec<D>, p: &mut Pool) -> Box<Self> {
    match root.key.cmp(&key) {
      Ordering::Equal => {
        root.data.as_mut().expect("Tree error: invalid node").push(data);
        return root;
      },
      Ordering::Less =>    root.node_right = Self::insert_into_node(key, data, root.node_right.take(), node_new, new, p),
      Ordering::Greater => root.node_left  = Self::insert_into_node(key, data, root.node_left.take(), node_new, new, p),
    }
    Self::update_height(&mut *root);
    Self::rotate_correctly(root)
  }

  fn updated_node(mut root: Box<Self>) -> Box<Self> {
    Self::update_height(&mut root);
    Self::rotate_correctly(root)
  }

  fn drop_min_from_left(mut root: Box<Self>, left: Box<Self>) -> (Option<Box<Self>>, Box<Self>) {
    let (new_left, min) = Self::drop_min(left);
    root.node_left = new_left;
    (Some(Self::updated_node(root)), min)
  }

  fn drop_min(mut root: Box<Self>) -> (Option<Box<Self>>, Box<Self>) {
    match root.node_left.take() {
      Some(left) => Self::drop_min_from_left(root, left),
      None => (root.node_right.take(), root),
    }
  }

  fn combine_branches(l: Box<Self>, r: Box<Self>) -> Box<Self> {
    let (trunc_right, min) = Self::drop_min(r);
    let mut newroot = min;
    newroot.node_left = Some(l);
    newroot.node_right = trunc_right;
    Self::updated_node(newroot)
  }

  fn delete(mut root: Box<Self>, drop_node: fn(&mut Pool, Box<Self>), p: &mut Pool) -> Option<Box<Self>> {
    let branches = (root.node_left.take(), root.node_right.take());
    drop_node(p, root);
    match branches {
      (None, None) => None,
      (Some(l), None) => Some(l),
      (None, Some(r)) => Some(r),
      (Some(l), Some(r)) => Some(Self::combine_branches(l, r)),
    }
  }

  fn pop_from_root(root: &mut Option<Box<Self>>, drop_node: fn(&mut Pool, Box<Self>), free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    let Some(rootnode) = root else { return None };
    let vec = rootnode.data.as_mut().expect("Tree error: invalid node");
    let retval = vec.pop();
    if vec.len() == 0 {
      free(p, rootnode.data.take().expect("Tree error: invalid node"));
      *root = Self::delete(root.take().unwrap(), drop_node, p);
    }
    retval
  }

  fn remove_at_least(key: K, root: &mut Option<Box<Self>>, drop_node: fn(&mut Pool, Box<Self>), free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    let Some(rootnode) = root else { return None };
    if rootnode.key.cmp(&key).is_ge() {
      Self::pop_from_root(root, drop_node, free, p)
    } else {
      let retval = Self::remove_at_least(key, &mut rootnode.node_right, drop_node, free, p);
      *root = Some(Self::updated_node(root.take().expect("Unreachable in Node::remove_at_least()")));
      retval
    }
  }

  fn remove_least_at_least(key: K, root: &mut Option<Box<Self>>, drop_node: fn(&mut Pool, Box<Self>), free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<D> {
    let Some(rootnode) = root else { return None };
    match rootnode.key.cmp(&key) {
      Ordering::Equal => {
        Self::pop_from_root(root, drop_node, free, p)
      },
      Ordering::Less => {
        let retval = Self::remove_at_least(key, &mut rootnode.node_right, drop_node, free, p);
        *root = Some(Self::updated_node(root.take().expect("Unreachable in Node::remove_at_least()")));
        retval
      },
      Ordering::Greater => {
        if rootnode.node_left.is_none() { return Self::pop_from_root(root, drop_node, free ,p) };
        let retval = Self::remove_at_least(key, &mut rootnode.node_left, drop_node, free, p);
        if retval.is_none() { return Self::pop_from_root(root, drop_node, free, p) };
        *root = Some(Self::updated_node(root.take().expect("Unreachable in Node::remove_at_least()")));
        retval
      },
    }
  }

  fn gc(mut root: Box<Self>, free: fn(&mut Pool, Vec<D>), p: &mut Pool) -> Option<Box<Self>> {
    match (root.node_left.take(), root.node_right.take()) {
      (Some(l), Some(r)) => {
        root.node_left = Self::gc(l, free, p);
        root.node_right = Self::gc(r, free, p);
      },
      (Some(l), None) => root.node_left = Self::gc(l, free, p),
      (None, Some(r)) => root.node_right = Self::gc(r, free, p),
      (None, None) => {
        free(p, root.data.take().expect("Tree error: invalid node"));
        return None
      }
    }
    root.update_height();
    Some(root)
  }
}

impl<D> Tree<usize,D> {
  pub fn size(&self) -> usize { Node::size(&self.root) }

  pub fn set_size(&mut self, size: usize, default_capacity: usize, create: fn (usize) -> D, node_new: fn(&mut Pool, usize, Vec<D>) -> Box<Node<usize,D>>,
                  new: fn(&mut Pool) -> Vec<D>, free: fn(&mut Pool, Vec<D>), p: &mut Pool) {
    let mut current_size = self.size();
    if size < current_size {
      loop {
        self.gc(free, p);
        current_size = self.size();
        if size >= current_size { return }
      }
    }
    if self.root.is_none() {
      let vec = new(p);
      self.root = Some(node_new(p, default_capacity, vec));
    }
    let root = self.root.as_mut().unwrap();
    let capacity = root.key;
    let diff = (size - current_size) / capacity;
    for _ in 0..diff {
      root.data.as_mut().expect("Tree error: invalid node").push(create(capacity))
    }
  }
}

impl<D> Node<usize,D> {
  fn size(node: &Option<Box<Self>>) -> usize {
    let Some(n) = node else { return 0 };
    let contrib = if let Some(ref d) = n.data { d.len() * n.key } else { 0 };
    Self::size(&n.node_left) + contrib + Self::size(&n.node_right)
  }
}
