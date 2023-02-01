use echo_core::Request;

pub fn extension<T>(req: &Request) -> Option<&T>
where
    T: Send + Sync + 'static,
{
    req.extensions().get::<T>()
}

pub fn extension_mut<T>(req: &mut Request) -> Option<&mut T>
where
    T: Send + Sync + 'static,
{
    req.extensions_mut().get_mut::<T>()
}
