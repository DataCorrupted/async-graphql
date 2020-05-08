use std::cmp::Ordering;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// Original position of element in source code
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Default, Hash)]
pub struct Pos {
    /// One-based line number
    pub line: usize,

    /// One-based column number
    pub column: usize,
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pos({}:{})", self.line, self.column)
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

#[derive(Clone, Debug, Copy)]
pub struct Spanned<T> {
    pub span: Span,
    pub node: T,
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> Spanned<T> {
    pub(crate) fn new(node: T, pair_span: pest::Span<'_>) -> Spanned<T> {
        let ((start_line, start_column), (end_line, end_column)) = (
            pair_span.start_pos().line_col(),
            pair_span.end_pos().line_col(),
        );
        Spanned {
            node,
            span: Span {
                start: Pos {
                    line: start_line,
                    column: start_column,
                },
                end: Pos {
                    line: end_line,
                    column: end_column,
                },
            },
        }
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    #[inline]
    pub fn pack<F: FnOnce(Self) -> R, R>(self, f: F) -> Spanned<R> {
        Spanned {
            span: self.span(),
            node: f(self),
        }
    }

    #[inline]
    pub fn map<F: FnOnce(T) -> R, R>(self, f: F) -> Spanned<R> {
        Spanned {
            span: self.span(),
            node: f(self.node),
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
