use langcities_common::error::Error;

use crate::{api::api_main, manager::messaging::DcManagerMessagingBuilder, state::AppState};

pub async fn aio_main(state: AppState) -> Result<(), Error> {
    let messaging_builder = DcManagerMessagingBuilder::new();
    let (_api_msg, _rxs, _txs, _worker_msg) = messaging_builder.build()?;
    let api_handle = tokio::spawn(api_main(state.clone()));
    let (api_result,) = tokio::join!(api_handle);
    api_result??;
    Ok(())
}
