use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Copy)]
pub struct Spanned<T> {
    pub span: Span,
    pub node: T,
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> Spanned<T> {
    pub fn new(node: T, start: usize, end: usize) -> Spanned<T> {
        Spanned {
            node,
            span: Span { start, end },
        }
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    #[inline]
    pub fn start_pos(&self) -> usize {
        self.span.start
    }

    #[inline]
    pub fn with<Q>(&self, node: Q) -> Spanned<Q> {
        Spanned {
            node,
            span: self.span,
        }
    }
}

impl<T: PartialOrd> PartialOrd for Spanned<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T: Ord> Ord for Spanned<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.node.cmp(&other.node)
    }
}

impl<T: Ord> Eq for Spanned<T> {}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}
