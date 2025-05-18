mod utils;

use std::fmt;
use std::fmt::Write;
use std::ptr::{null, null_mut};

#[derive(PartialEq, Copy, Clone)]
enum NodeDirection {
    LeftChild = 0,
    RightChild = 1,
}

impl From<NodeDirection> for usize {
    fn from(child: NodeDirection) -> usize {
        match child {
            NodeDirection::LeftChild => 0,
            NodeDirection::RightChild => 1,
        }
    }
}

impl From<bool> for NodeDirection {
    fn from(branch: bool) -> NodeDirection {
        match branch {
            false => Self::LeftChild,
            true => Self::RightChild,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum NodeColor {
    Red,
    Black,
}

impl fmt::Display for NodeColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeColor::Red => write!(f, "R"),
            NodeColor::Black => write!(f, "B"),
        }
    }
}

pub trait RbTrait<Key> {
    fn new(key: Key) -> Self;
    fn get(&self) -> Key;
    fn set(&mut self, key: Key);
}

pub struct RbNode<Key: Default, T: RbTrait<Key> + PartialOrd + fmt::Display> {
    color: NodeColor,
    val: T,
    parent: *mut RbNode<Key, T>,
    childs: [*mut RbNode<Key, T>; 2],
}

impl<Key: Default, T: RbTrait<Key> + PartialOrd + fmt::Display> RbNode<Key, T> {
    pub fn new(key: Key) -> Self {
        RbNode {
            color: NodeColor::Red,
            val: T::new(key),
            parent: null_mut(),
            childs: [null_mut(), null_mut()],
        }
    }

    pub fn set(&mut self, key: Key) {
        self.val.set(key)
    }

    #[inline]
    fn get_child(&self, child: NodeDirection) -> *mut RbNode<Key, T> {
        self.childs[usize::from(child)]
    }

    #[inline]
    fn has_red_child(&self) -> bool {
        let left = self.get_child(NodeDirection::LeftChild);
        let right = self.get_child(NodeDirection::RightChild);
        if !left.is_null() {
            unsafe {
                if (*left).color == NodeColor::Red {
                    return true;
                }
            }
        }

        if !right.is_null() {
            unsafe {
                if (*right).color == NodeColor::Red {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    fn is_black(&self) -> bool {
        self.color == NodeColor::Black
    }

    #[inline]
    fn is_red(&self) -> bool {
        self.color == NodeColor::Red
    }

    #[inline]
    fn get_parent(&self) -> *mut RbNode<Key, T> {
        self.parent
    }

    #[inline]
    fn set_color(&mut self, color: NodeColor) {
        self.color = color
    }

    #[inline]
    fn get_direction(&mut self, parent: *mut RbNode<Key, T>) -> NodeDirection {
        unsafe {
            if (*parent).childs[usize::from(NodeDirection::LeftChild)]
                == self as *mut RbNode<Key, T>
            {
                NodeDirection::LeftChild
            } else {
                NodeDirection::RightChild
            }
        }
    }

    #[inline]
    fn get_the_other_child_of_color(
        &self,
        di: &NodeDirection,
        color: NodeColor,
    ) -> *mut RbNode<Key, T> {
        let sel = match di {
            NodeDirection::LeftChild => 1,
            NodeDirection::RightChild => 0,
        };

        let child = self.childs[sel];

        return if child.is_null() {
            null_mut::<RbNode<Key, T>>()
        } else {
            unsafe {
                if (*child).color == color {
                    child
                } else {
                    null_mut::<RbNode<Key, T>>()
                }
            }
        };
    }

    #[inline]
    fn insert_child(&mut self, di: NodeDirection, node: *mut RbNode<Key, T>) {
        self.childs[usize::from(di)] = node;
        unsafe { (*node).parent = self }
    }

    #[inline]
    fn set_child(&mut self, child: *mut RbNode<Key, T>, di: NodeDirection, color: NodeColor) {
        self.childs[usize::from(di)] = child;
        unsafe {
            if !child.is_null() {
                (*child).parent = self;
                (*child).color = color
            }
        }
    }

    #[inline]
    fn set_child_without_color(&mut self, child: *mut RbNode<Key, T>, di: NodeDirection) {
        self.childs[usize::from(di)] = child;
        if !child.is_null() {
            unsafe { (*child).parent = self }
        }
    }

    #[inline]
    fn rotate_with_parent(
        &mut self,
        parent: *mut RbNode<Key, T>,
        di: NodeDirection,
        color: NodeColor,
    ) {
        let other = if di == NodeDirection::LeftChild {
            NodeDirection::RightChild
        } else {
            NodeDirection::LeftChild
        };
        unsafe { (*parent).set_child_without_color(self.childs[usize::from(other)], di) }
        self.set_child(parent, other, color)
    }

    #[inline]
    fn inherit_parent(&mut self, node: *mut RbNode<Key, T>, tree: &mut RbTree<Key, T>) {
        unsafe {
            let parent = (*node).parent;
            self.parent = parent;
            self.color = (*node).color;
            if parent.is_null() {
                tree.root = self;
            } else {
                let di = (*node).get_direction(parent);
                (*parent).childs[usize::from(di)] = self;
            }
        }
    }

    #[inline]
    fn hook_old_child(&mut self, child: *mut RbNode<Key, T>, di: NodeDirection) {
        self.childs[usize::from(di)] = child;
        if !child.is_null() {
            unsafe { (*child).parent = self }
        }
    }
}

const INITIAL_BLACK_COUNTER: i32 = -1;

pub struct RbTree<Key: Default, T: RbTrait<Key> + PartialOrd + fmt::Display> {
    root: *mut RbNode<Key, T>,
}

impl<Key: Default, T: RbTrait<Key> + PartialOrd + fmt::Display> RbTree<Key, T> {
    pub fn new() -> Self {
        RbTree { root: null_mut() }
    }

    pub fn insert(&mut self, node: &mut RbNode<Key, T>) -> &mut Self {
        let mut parent: *mut RbNode<Key, T> = null_mut();
        let mut p = self.root;
        let mut branch: bool = false;

        while !p.is_null() {
            parent = p;
            unsafe {
                branch = (*p).val.lt(&(*node).val);
                p = (*p).get_child(NodeDirection::from(branch));
            }
        }

        if parent.is_null() {
            // root node
            self.root = node;
            node.color = NodeColor::Black
        } else {
            unsafe { (*parent).insert_child(NodeDirection::from(branch), node) }
            self.insert_rebalance(node)
        }

        #[cfg(test)]
        assert!(self.verify_tree());
        self
    }

    fn insert_rebalance(&mut self, mut node: *mut RbNode<Key, T>) {
        let mut p: *mut RbNode<Key, T>;
        let mut gp: *mut RbNode<Key, T>;
        let mut nd: NodeDirection;
        let mut pd: NodeDirection;

        loop {
            p = unsafe { (*node).get_parent() };
            if p.is_null() {
                unsafe { (*node).set_color(NodeColor::Black) };
                self.root = node;
                break;
            }

            if unsafe { (*p).is_black() } {
                break;
            }

            gp = unsafe { (*p).get_parent() };
            nd = unsafe { (*node).get_direction(p) };
            pd = unsafe { (*p).get_direction(gp) };
            let uncle = unsafe { (*gp).get_the_other_child_of_color(&pd, NodeColor::Red) };
            if !uncle.is_null() {
                unsafe {
                    (*p).set_color(NodeColor::Black);
                    (*uncle).set_color(NodeColor::Black);
                    node = gp;
                    (*node).set_color(NodeColor::Red);
                    continue;
                }
            } else {
                if nd != pd {
                    unsafe { (*node).rotate_with_parent(p, nd, NodeColor::Red) }
                    nd = pd;
                    p = node;
                }

                unsafe {
                    (*p).inherit_parent(gp, self);
                    (*p).rotate_with_parent(gp, nd, NodeColor::Red)
                }
                break;
            }
        }
    }

    pub fn delete(&mut self, node: &mut RbNode<Key, T>) -> &mut Self {
        let left = node.childs[0];
        let right = node.childs[1];
        let mut di = NodeDirection::LeftChild;

        let mut fix = null_mut::<RbNode<Key, T>>();

        if left.is_null() {
            if !right.is_null() {
                unsafe { (*right).inherit_parent(node, self) }
            } else {
                let p = node.get_parent();
                if p.is_null() {
                    self.root = null_mut()
                } else {
                    di = node.get_direction(p);
                    unsafe {
                        (*p).childs[usize::from(di)] = null_mut();
                    }
                    if node.is_black() {
                        fix = p;
                    }
                }
            }
        } else if right.is_null() {
            unsafe { (*left).inherit_parent(node, self) }
        } else {
            let mut far_left: *mut RbNode<Key, T>;
            let near_right: *mut RbNode<Key, T>;
            let mut x = right;

            loop {
                far_left = x;
                unsafe { x = (*x).childs[0] }
                if x.is_null() {
                    break;
                }
            }

            near_right = unsafe { (*far_left).childs[1] };
            let need_fix = unsafe { (*far_left).is_black() && near_right.is_null() };
            if far_left != right {
                unsafe {
                    fix = (*far_left).parent;
                    (*fix).set_child(near_right, NodeDirection::LeftChild, NodeColor::Black);
                    (*far_left).hook_old_child(right, NodeDirection::RightChild);
                }
            } else {
                fix = far_left;
                di = NodeDirection::RightChild;
                if !near_right.is_null() {
                    unsafe { (*near_right).color = NodeColor::Black }
                }
            }

            if !need_fix {
                fix = null_mut();
            };

            unsafe {
                (*far_left).inherit_parent(node, self);
                (*far_left).hook_old_child(left, NodeDirection::LeftChild);
            }
        }

        if !fix.is_null() {
            self.delete_reblance(fix, di)
        }

        #[cfg(test)]
        assert!(self.verify_tree());
        self
    }

    fn delete_reblance(&mut self, mut parent: *mut RbNode<Key, T>, mut nd: NodeDirection) {
        let mut node: *mut RbNode<Key, T>;
        unsafe {
            loop {
                let sd = if nd == NodeDirection::LeftChild {
                    NodeDirection::RightChild
                } else {
                    NodeDirection::LeftChild
                };
                let mut s = (*parent).childs[usize::from(sd)];
                let color = if s.is_null() {
                    NodeColor::Black
                } else {
                    (*s).color
                };

                if color == NodeColor::Red {
                    (*s).inherit_parent(parent, self);
                    (*s).rotate_with_parent(parent, sd, NodeColor::Red);
                    s = (*parent).childs[usize::from(sd)];
                }

                let left = (*s).childs[0];
                let left_color = if left.is_null() {
                    NodeColor::Black
                } else {
                    (*left).color
                };
                let right = (*s).childs[1];
                let right_color = if right.is_null() {
                    NodeColor::Black
                } else {
                    (*right).color
                };

                let sc = [left, right];
                let scc = [left_color, right_color];

                if left_color == NodeColor::Black && right_color == NodeColor::Black {
                    (*s).color = NodeColor::Red;
                    node = parent;
                    if node != self.root && (*node).is_black() {
                        nd = (*node).get_direction((*node).parent);
                        parent = (*node).parent;
                        continue;
                    } else {
                        break;
                    }
                } else {
                    if scc[usize::from(sd)] == NodeColor::Black {
                        (*sc[usize::from(nd)]).inherit_parent(s, self);
                        (*sc[usize::from(nd)]).rotate_with_parent(s, nd, NodeColor::Red);
                        s = (*s).parent;
                    }

                    (*s).inherit_parent(parent, self);
                    (*s).rotate_with_parent(parent, sd, NodeColor::Black);
                    let sdc = (*s).childs[usize::from(sd)];
                    if !sdc.is_null() {
                        (*sdc).color = NodeColor::Black;
                    }
                    node = self.root;
                    break;
                }
            }
            (*node).color = NodeColor::Black;
        }
    }

    pub fn traversal_preorder(&self, node: *mut RbNode<Key, T>, f: fn(*mut RbNode<Key, T>)) {
        if !node.is_null() {
            f(node);
            self.traversal_preorder(unsafe { (*node).childs[0] }, f);
            self.traversal_preorder(unsafe { (*node).childs[1] }, f);
        }
    }

    pub fn dump_tree(&self) {
        self.traversal_preorder(self.root, |node| unsafe {
            let parent = (*node).parent;
            let left = (*node).childs[0];
            let right = (*node).childs[1];

            let mut out = String::new();
            let _ = write!(out, "{}({}): parent: ", (*node).color, (*node).val);
            if parent.is_null() {
                let _ = write!(out, "nil, left: ");
            } else {
                let _ = write!(out, "{}, left: ", (*parent).val);
            }

            if left.is_null() {
                let _ = write!(out, "nil, right: ");
            } else {
                let _ = write!(out, "{}, right: ", (*left).val);
            }

            if right.is_null() {
                let _ = write!(out, "nil");
            } else {
                let _ = write!(out, "{}", (*right).val);
            }

            println!("{}", out);
        })
    }

    fn verify_properties(
        &self,
        node: *mut RbNode<Key, T>,
        black_count: &mut i32,
        mut current_black_count: i32,
    ) -> bool {
        if node.is_null() {
            if *black_count == INITIAL_BLACK_COUNTER {
                *black_count = current_black_count;
            }
            return *black_count == current_black_count;
        }

        unsafe {
            if (*node).is_red() {
                if (*node).has_red_child() {
                    return false;
                }
            } else {
                current_black_count += 1
            }
        }

        unsafe {
            self.verify_properties((*node).childs[0], black_count, current_black_count)
                && self.verify_properties((*node).childs[1], black_count, current_black_count)
        }
    }

    fn bst_traversal(
        node: *const RbNode<Key, T>,
        left: *const RbNode<Key, T>,
        right: *const RbNode<Key, T>,
    ) -> bool {
        unsafe {
            if !node.is_null() {
                if !left.is_null() && (*node).val < (*left).val {
                    return false;
                }
                if !right.is_null() && (*node).val > (*right).val {
                    return false;
                }
                Self::bst_traversal((*node).childs[0], left, node)
                    && Self::bst_traversal((*node).childs[1], node, right)
            } else {
                true
            }
        }
    }

    fn verify_bst(&self) -> bool {
        Self::bst_traversal(
            self.root,
            null::<RbNode<Key, T>>(),
            null::<RbNode<Key, T>>(),
        )
    }

    pub fn verify_tree(&self) -> bool {
        let mut count = INITIAL_BLACK_COUNTER;
        if self.root.is_null() {
            return true;
        }

        self.verify_properties(self.root, &mut count, 0) && self.verify_bst()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils;

    #[derive(PartialOrd, PartialEq)]
    struct Test {
        key: i32,
    }

    impl RbTrait<i32> for Test {
        fn new(key: i32) -> Self {
            Test { key }
        }

        fn get(&self) -> i32 {
            self.key
        }

        fn set(&mut self, key: i32) {
            self.key = key
        }
    }

    impl fmt::Display for Test {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.key)
        }
    }

    #[test]
    fn stress() {
        let mut array = [0i32; 1000];
        let mut tree = RbTree::<i32, Test>::new();
        let mut nodes: [RbNode<i32, Test>; 1000] =
            std::array::from_fn(|_| RbNode::<i32, Test>::new(0));

        let _ = utils::read_blocks_from_file::<i32>("/dev/urandom", &mut array, 1000);
        for i in 0..1000 {
            nodes[i].set(array[i]);
            tree.insert(&mut nodes[i]);
        }

        for i in 0..1000 {
            tree.delete(&mut nodes[i]);
        }
    }

    #[test]
    fn debug() {
        let mut a = RbNode::<i32, Test>::new(1);
        let mut b = RbNode::<i32, Test>::new(3);
        let mut c = RbNode::<i32, Test>::new(8);
        let mut d = RbNode::<i32, Test>::new(6);
        let mut e = RbNode::<i32, Test>::new(5);
        let mut f = RbNode::<i32, Test>::new(10);
        let mut g = RbNode::<i32, Test>::new(-1);
        let mut h = RbNode::<i32, Test>::new(158);
        let mut i = RbNode::<i32, Test>::new(10);
        let mut j = RbNode::<i32, Test>::new(166);
        let mut k = RbNode::<i32, Test>::new(-56);
        let mut l = RbNode::<i32, Test>::new(28);
        let mut m = RbNode::<i32, Test>::new(-158);

        let mut tree = RbTree::<i32, Test>::new();

        tree.insert(&mut a)
            .insert(&mut b)
            .insert(&mut c)
            .insert(&mut d)
            .insert(&mut e)
            .insert(&mut f)
            .insert(&mut g)
            .insert(&mut h)
            .insert(&mut i)
            .insert(&mut j)
            .insert(&mut k)
            .insert(&mut l)
            .insert(&mut m);

        assert!(tree.verify_tree());

        tree.dump_tree();

        tree.delete(&mut a)
            .delete(&mut d)
            .delete(&mut h)
            .delete(&mut m)
            .delete(&mut k)
            .delete(&mut b)
            .delete(&mut i)
            .delete(&mut c)
            .delete(&mut j)
            .delete(&mut l)
            .delete(&mut e)
            .delete(&mut g)
            .delete(&mut f);

        tree.dump_tree();
    }
}
