pub struct TreeNode<T> {
    data: T,
    children: Vec<Self>,
}

impl<T> TreeNode<T> {
    pub const fn root(root_data: T) -> TreeNode<T> {
        Self {
            data: root_data,
            children: vec![],
        }
    }

    pub const fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn children(&self) -> &[Self] {
        self.children.as_slice()
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(child);
    }
}
