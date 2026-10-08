use std::pin::Pin;

use langcities_common::error::Error;
use tokio::sync::broadcast::{Sender, channel};

use crate::{
    api::{
        api_main_with_state,
        state::{ApiState, OuterApiState},
    },
    config::Config,
    error::DcAppError,
    hub::Hub,
    manager::Manager,
    message::builder::MxBuilder,
    state::AppState,
};

pub async fn generate_states(config: Config) -> Result<(ApiState, Hub, Manager), DcAppError> {
    // TODO: configure mx_builder
    let mx_builder = MxBuilder::new();
    let (api_mxs, rxs, txs, relay_mxs) = mx_builder.build()?;
    let outer_api_state = OuterApiState::create(&config, api_mxs)?;
    let app_state = AppState::create(config).await?;
    let hub = Hub::new(app_state.clone(), rxs, txs);
    let manager = Manager::create(app_state.clone(), relay_mxs)?;
    let api_state = ApiState::new(app_state, outer_api_state);
    Ok((api_state, hub, manager))
}

fn stop_rx(stop_tx: &Sender<()>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    let mut stop_rx = stop_tx.subscribe();
    Box::pin(async move {
        let _ = stop_rx.recv().await;
        ()
    })
}

pub async fn aio_main(
    config: Config,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> Result<(), Error> {
    let (api_state, dcm, manager) = generate_states(config).await?;
    let (stop_tx, _) = channel::<()>(1);
    let api_handle = tokio::spawn(api_main_with_state(api_state.clone(), stop_rx(&stop_tx)));
    let hub_handle = tokio::spawn(dcm.run(stop_rx(&stop_tx)));
    let manager_handle = tokio::spawn(manager.run(stop_rx(&stop_tx)));
    let stop_handle = tokio::spawn(async move {
        shutdown_signal.await;
        let _ = stop_tx.send(());
    });
    let (api_result, hub_result, manager_result, stop_result) =
        tokio::join!(api_handle, hub_handle, manager_handle, stop_handle);
    api_result??;
    hub_result?;
    manager_result??;
    stop_result?;
    Ok(())
}
