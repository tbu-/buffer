struct SliceBuffer<'data> {
    // TODO: Double mut.
    slice: &'data mut [u8],
    initialized: usize,
}
