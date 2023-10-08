//! Singly-linked stack data structure.

/// Single-linked stack.
#[repr(transparent)]
pub struct Stack<T> {
    root: Option<Box<Node<T>>>,
}

struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack { root: None }
    }

    pub fn is_empty(&self) -> bool {
        return self.root.is_none();
    }

    pub fn push(&mut self, data: T) {
        let next = self.root.take();
        self.root = Some(Box::new(Node { data, next }))
    }

    pub fn pop(&mut self) -> Option<T> {
        self.root.take().map(|n| {
            self.root = n.next;
            n.data
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.root.as_ref().map(|n| &n.data)
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Stack::new()
    }
}

#[test]
fn test_empty() {
    let mut stack: Stack<i32> = Stack::new();
    assert!(stack.is_empty());
    assert_eq!(stack.peek(), None);
    assert_eq!(stack.pop(), None);
}

#[test]
fn test_push_one() {
    let mut stack = Stack::new();
    stack.push(3);
    assert!(!stack.is_empty());
    assert_eq!(stack.peek(), Some(&3));
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.peek(), None);
    assert_eq!(stack.pop(), None);
}
