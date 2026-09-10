use crate::substrate::CrunchConfig;
use subxt::{
    client::OnlineClientAtBlockImpl,
    config::ParamsFor,
    tx::{Payload, Signer, TransactionProgress, TransactionsClient},
    Error as SubxtError,
};

/// crunch always signs and submits a V5 "General" transaction rather than relying on
/// subxt's default (V4) auto-selection.
///
/// Reason: subxt/frame-decode currently pick the *highest* transaction-extension pipeline
/// version even when encoding a V4 extrinsic (`transaction_extension_version_to_use_for_decoding`
/// is reused for V4 encoding, and it always takes the max key). On chains that publish more
/// than one pipeline version (e.g. asset-hub-polkadot, which added a second pipeline for its
/// `pallet_verify_signature`/individuality extensions), that mismatches the 13-extension
/// baseline the runtime actually expects for a V4-tagged extrinsic, corrupting the signed-extra
/// bytes and crashing the runtime with an `unreachable` trap during `validate_transaction`.
///
/// V5 doesn't have this bug, since it's *meant* to always use the highest pipeline version.
/// It's also safe on chains with only one pipeline (kusama/paseo/westend asset-hubs today):
/// there, V5 just selects the same single pipeline V4 would have used.
pub async fn sign_and_submit_v5_then_watch<Call, S>(
    tx_client: &TransactionsClient<CrunchConfig, OnlineClientAtBlockImpl<CrunchConfig>>,
    call: &Call,
    signer: &S,
    params: ParamsFor<CrunchConfig>,
) -> Result<
    TransactionProgress<CrunchConfig, OnlineClientAtBlockImpl<CrunchConfig>>,
    SubxtError,
>
where
    Call: Payload,
    S: Signer<CrunchConfig>,
{
    let mut signable = tx_client
        .create_v5_signable(call, &signer.account_id(), params)
        .await?;
    let submittable = signable.sign(signer)?;
    submittable.submit_and_watch().await.map_err(Into::into)
}
