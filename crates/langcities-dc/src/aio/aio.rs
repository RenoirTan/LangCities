use langcities_common::error::Error;

use crate::{
    api::{
        api_main_with_state,
        state::{ApiState, OuterApiState},
    },
    config::Config,
    error::DcAppError,
    hub::Hub,
    message::builder::MxBuilder,
    state::AppState,
};

pub async fn generate_states(config: Config) -> Result<(ApiState, Hub), DcAppError> {
    // TODO: configure mx_builder
    let mx_builder = MxBuilder::new();
    let (api_mx, rxs, txs, worker_mx) = mx_builder.build()?;
    let outer_api_state = OuterApiState::create(&config, api_mx)?;
    let app_state = AppState::create(config).await?;
    let hub = Hub::new(app_state.clone(), rxs, txs, worker_mx);
    let api_state = ApiState::new(app_state, outer_api_state);
    Ok((api_state, hub))
}

pub async fn aio_main(config: Config) -> Result<(), Error> {
    let (api_state, dcm) = generate_states(config).await?;
    let api_handle = tokio::spawn(api_main_with_state(api_state.clone()));
    let hub_handle = tokio::spawn(dcm.run());
    let (api_result, hub_result) = tokio::join!(api_handle, hub_handle);
    api_result??;
    hub_result?;
    Ok(())
}
