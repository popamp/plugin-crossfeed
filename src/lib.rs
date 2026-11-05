//! Crossfeed for Popamp: a dsp plug-in that makes stereo on headphones
//! sound less like it is inside your head. The calls are described in
//! [popamp-plugin-api](https://github.com/popamp/popamp-plugin-api).

pub mod crossfeed;

#[cfg(target_arch = "wasm32")]
mod plugin {
    use crate::crossfeed::{Crossfeed, Settings};
    use extism_pdk::*;
    use popamp_plugin_api::{bytes_of, samples_of, Format, Version, VERSION};
    use std::cell::RefCell;

    thread_local! {
        /// Set by `reset`, which the player calls before the first block;
        /// cleared when the settings are wrong.
        static CROSSFEED: RefCell<Option<Crossfeed>> = const { RefCell::new(None) };
    }

    #[plugin_fn]
    pub fn api_version(_: String) -> FnResult<Json<Version>> {
        Ok(Json(VERSION))
    }

    #[plugin_fn]
    pub fn reset(Json(format): Json<Format>) -> FnResult<()> {
        let level = config::get("level")?.unwrap_or_default();
        let cutoff = config::get("cutoff")?.unwrap_or_default();
        CROSSFEED.set(None);
        let settings = Settings::parse(&level, &cutoff).map_err(|e| WithReturnCode::new(Error::msg(e), 1))?;
        CROSSFEED.set(Some(Crossfeed::new(settings, format.rate)));
        Ok(())
    }

    #[plugin_fn]
    pub fn process(block: Vec<u8>) -> FnResult<Vec<u8>> {
        let mut samples = samples_of(&block);
        // Until a reset with good settings, the sound stays as it is.
        CROSSFEED.with_borrow_mut(|cf| cf.as_mut().map(|cf| cf.process(&mut samples)));
        Ok(bytes_of(&samples))
    }
}
