use crate::repo::location_reading::LocationReadingRepo;
use crate::warp_ext;
use crate::warp_ext::BoxReply;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

#[derive(Clone)]
pub struct DeviceDataContext {
    pub location_reading_repo: Arc<dyn LocationReadingRepo + Send + Sync + 'static>,
}

pub fn device_data_filter(context: DeviceDataContext) -> BoxedFilter<(Box<dyn Reply>,)> {
    warp::post()
        .and(warp::path("import"))
        .and(warp::path("deviceData"))
        .and(warp_ext::with_clone(context))
        .then(move |context: DeviceDataContext| async move { Ok(warp::reply().boxed()) })
        .map(warp_ext::convert_err)
        .boxed()
}
