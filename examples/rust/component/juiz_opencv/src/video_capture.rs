

use juiz_core::{prelude::*, opencv::videoio::*};

#[allow(dead_code)]
#[repr(Rust)]
pub struct CvVideoCapture {
    pub camera: VideoCapture
}

impl CvVideoCapture {

    pub fn manifest() -> Value {
        ContainerManifest::new("cv_video_capture").into()
    }
}

impl Drop for CvVideoCapture {
    fn drop(&mut self) {
        log::info!("CvVideoCapture::drop() called");
    }
}

fn create_cv_capture_container(_manifest: Value) -> JuizResult<Box<CvVideoCapture>> {
    log::trace!("create_cv_capture_container({}) called", _manifest);
    let cam = VideoCapture::new(0, CAP_ANY)?; // 0 is the default camera
    Ok(Box::new(CvVideoCapture{camera: cam}))
}

#[no_mangle]
pub unsafe extern "Rust" fn cv_video_capture_factory() -> JuizResult<ContainerFactoryPtr> {
    log::trace!("cv_video_capture_factory() called");
    ContainerFactoryImpl::create(CvVideoCapture::manifest(), create_cv_capture_container)
}
