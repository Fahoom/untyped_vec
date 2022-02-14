use std::alloc::Layout;

pub(super) fn array_layout(layout: &Layout, amount: usize) -> Option<Layout> {
    let (array_layout, offset) = repeat_layout(layout, amount)?;
    debug_assert_eq!(layout.size(), offset);
    Some(array_layout)
}

pub(super) fn repeat_layout(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
    let padded_size = layout.size() + padding_needed_for(layout, layout.align());
    let alloc_size = padded_size.checked_mul(n)?;

    unsafe {
        Some((
            Layout::from_size_align_unchecked(alloc_size, layout.align()),
            padded_size,
        ))
    }
}

pub(super) const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let len = layout.size();
    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}

pub(super) const fn to_const_ptr<T>(val: &T) -> *const u8 {
    (val as *const T).cast::<u8>()
}

// SAFETY : x must point to valid data!
pub(super) unsafe fn drop_ptr<T>(x: *mut u8) {
    x.cast::<T>().drop_in_place()
}

pub(super) fn convert_ptr<T>(ptr: *const u8) -> *const T {
    ptr.cast::<T>()
}
