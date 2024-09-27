//! Id sizes, retrieved from the VM

/// Contains the data on the size of ids in use by the target JVM
#[derive(Debug, Copy, Clone)]
pub struct IdSizes {
    object_id_size: usize,
    method_id_size: usize,
    field_id_size: usize,
    frame_id_size: usize,
}

impl IdSizes {
    /// Creates a new id sizes object
    pub fn new(
        object_id_size: usize,
        method_id_size: usize,
        field_id_size: usize,
        frame_id_size: usize,
    ) -> Self {
        Self {
            object_id_size,
            method_id_size,
            field_id_size,
            frame_id_size,
        }
    }

    /// Gets the size (in bytes) of object ids
    pub fn object_id_size(&self) -> usize {
        self.object_id_size
    }

    /// Gets the size (in bytes) of method ids
    pub fn method_id_size(&self) -> usize {
        self.method_id_size
    }

    /// Gets the size (in bytes) of field ids
    pub fn field_id_size(&self) -> usize {
        self.field_id_size
    }

    /// Gets the size (in bytes) of frame ids
    pub fn frame_id_size(&self) -> usize {
        self.frame_id_size
    }
}

impl Default for IdSizes {
    fn default() -> Self {
        Self {
            object_id_size: 8,
            method_id_size: 8,
            field_id_size: 8,
            frame_id_size: 8,
        }
    }
}
