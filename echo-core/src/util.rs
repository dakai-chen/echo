pub(crate) fn try_downcast<Src: 'static, Dst: 'static>(src: Src) -> Result<Dst, Src> {
    let mut src = Some(src);
    if let Some(dst) = <dyn std::any::Any>::downcast_mut::<Option<Dst>>(&mut src) {
        Ok(dst.take().unwrap())
    } else {
        Err(src.unwrap())
    }
}
