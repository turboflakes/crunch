use crate::substrate::CrunchConfig;
use crunch_config::CONFIG;
use crunch_error::CrunchError;
use log::{debug, warn};
use std::time::Duration;
use subxt::{
    client::{ClientAtBlock, OnlineClient, OnlineClientAtBlockImpl},
    config::ParamsFor,
    tx::{Payload, Signer, TransactionInBlock, TransactionProgress, TransactionStatus},
    Error as SubxtError,
};

type CrunchClientAtBlock =
    ClientAtBlock<CrunchConfig, OnlineClientAtBlockImpl<CrunchConfig>>;
type CrunchTransactionProgress =
    TransactionProgress<CrunchConfig, OnlineClientAtBlockImpl<CrunchConfig>>;

/// Whether a chain's metadata advertises more than one transaction-extension pipeline
/// version. Version 0 is always the baseline; a higher version existing means the chain
/// deliberately added extensions on top of it, including — so far, in practice — an
/// authorization extension that a V5 "General" transaction needs in order to carry a
/// signed origin. See [`sign_and_submit_then_watch`] for the full explanation.
pub fn should_use_v5_transaction(metadata: &subxt::Metadata) -> bool {
    metadata
        .extrinsic()
        .transaction_extension_version_to_use_for_encoding()
        > 0
}

/// Signs and submits a transaction, picking between the classic V4 "Signed" envelope and the
/// newer V5 "General" one depending on what the chain's metadata actually supports and needs.
///
/// # Why not always use one or the other?
///
/// subxt/frame-decode currently pick the *highest* transaction-extension pipeline version even
/// when encoding a V4 extrinsic (`transaction_extension_version_to_use_for_decoding` is reused
/// for V4 encoding, and it always takes the max key). On chains that publish more than one
/// pipeline version (e.g. asset-hub-polkadot, which added a second pipeline for its
/// `pallet_verify_signature`/individuality extensions), that mismatches the baseline pipeline
/// the runtime actually expects for a V4-tagged extrinsic, corrupting the signed-extra bytes and
/// crashing the runtime with an `unreachable` trap during `validate_transaction`. Forcing V5
/// there sidesteps the bug, since V5 is *meant* to always use the highest pipeline version.
///
/// But V5 isn't a safe universal default either: a V5 "General" transaction only carries a
/// signed origin if one of the extensions in the selected pipeline is an *authorization*
/// extension (like `VerifyMultiSignature`) that embeds the signature. Chains that haven't added
/// such an extension yet (asset-hub-kusama/paseo/westend today, which only have the single
/// baseline pipeline) reject a V5 submission with "the transaction extension did not authorize
/// any origin" — there's nothing in their extension set that can attach an origin outside of
/// the classic V4 envelope's own Address+Signature fields.
///
/// So: only reach for V5 when [`should_use_v5_transaction`] says the metadata supports it.
/// Otherwise stick to the classic, universally-supported V4 path.
#[allow(clippy::result_large_err)]
pub async fn sign_and_submit_then_watch<Call, S>(
    client_at_block: &CrunchClientAtBlock,
    call: &Call,
    signer: &S,
    params: ParamsFor<CrunchConfig>,
) -> Result<CrunchTransactionProgress, SubxtError>
where
    Call: Payload,
    S: Signer<CrunchConfig>,
{
    if should_use_v5_transaction(client_at_block.metadata_ref()) {
        let mut signable = client_at_block
            .transactions()
            .create_v5_signable(call, &signer.account_id(), params)
            .await?;
        let submittable = signable.sign(signer)?;
        submittable.submit_and_watch().await.map_err(Into::into)
    } else {
        client_at_block
            .transactions()
            .sign_and_submit_then_watch(call, signer, params)
            .await
            .map_err(Into::into)
    }
}

pub type CrunchTransactionInBlock =
    TransactionInBlock<CrunchConfig, OnlineClientAtBlockImpl<CrunchConfig>>;

/// How a watched transaction ended.
enum TxOutcome {
    /// Finalized in a block.
    Finalized(CrunchTransactionInBlock),
    /// The node reported `Error`, `Invalid` or `Dropped`: the transaction is not included.
    NotIncluded(String),
    /// The status stream failed or ended early (e.g. `chainHead_follow` stopped): the
    /// transaction may or may not have been included.
    Lost(String),
}

async fn watch_until_finalized(mut progress: CrunchTransactionProgress) -> TxOutcome {
    while let Some(status) = progress.next().await {
        match status {
            Ok(TransactionStatus::InFinalizedBlock(in_block)) => {
                return TxOutcome::Finalized(in_block)
            }
            Ok(TransactionStatus::Error { message }) => {
                return TxOutcome::NotIncluded(format!("Error: {message}"))
            }
            Ok(TransactionStatus::Invalid { message }) => {
                return TxOutcome::NotIncluded(format!("Invalid: {message}"))
            }
            Ok(TransactionStatus::Dropped { message }) => {
                return TxOutcome::NotIncluded(format!("Dropped: {message}"))
            }
            Ok(TransactionStatus::Broadcasted) => debug!("TxStatus: Broadcasted"),
            Ok(TransactionStatus::InBestBlock(in_block)) => {
                debug!("TxStatus: InBestBlock hash: {:?}", in_block.block_hash())
            }
            Ok(TransactionStatus::NoLongerInBestBlock) => {
                debug!("TxStatus: NoLongerInBestBlock")
            }
            Ok(TransactionStatus::Validated) => debug!("TxStatus: Validated"),
            Err(e) => return TxOutcome::Lost(e.to_string()),
        }
    }
    TxOutcome::Lost("transaction status stream ended unexpectedly".to_string())
}

/// Signs, submits and watches a transaction until it is finalized, retrying when it is safe to.
///
/// * `Ok(Some(in_block))` - finalized; the caller can read the events from `in_block`.
/// * `Ok(None)` - the status stream was lost (e.g. the node stopped `chainHead_follow`) but the
///   signer nonce shows the transaction was included. Its events could not be read.
/// * `Err(_)` - not included after `tx_max_submit_attempts` attempts.
///
/// A transaction the node reports as `Invalid`/`Dropped`/`Error` is resubmitted. When the status
/// stream is lost the transaction is never resubmitted blindly: the signer nonce is compared with
/// the one used for the attempt, and only if it did not advance is the transaction resubmitted.
pub async fn submit_and_wait_finalized<Call, S>(
    client: &OnlineClient<CrunchConfig>,
    call: &Call,
    signer: &S,
    params: impl Fn() -> ParamsFor<CrunchConfig>,
) -> Result<Option<CrunchTransactionInBlock>, CrunchError>
where
    Call: Payload,
    S: Signer<CrunchConfig>,
{
    let config = CONFIG.clone();
    let max_attempts = config.tx_max_submit_attempts.max(1);
    let settle_secs = config.tx_lost_settle_secs;
    let mut last_error = String::new();

    for attempt in 1..=max_attempts {
        let block = client.at_current_block().await?;
        let nonce = block
            .transactions()
            .account_nonce(&signer.account_id())
            .await?;

        let progress = sign_and_submit_then_watch(&block, call, signer, params()).await?;

        match watch_until_finalized(progress).await {
            TxOutcome::Finalized(in_block) => return Ok(Some(in_block)),
            TxOutcome::NotIncluded(message) => {
                warn!("TxStatus: {message} ({attempt}/{max_attempts})");
                last_error = message;
            }
            TxOutcome::Lost(message) => {
                warn!("TxStatus: lost ({attempt}/{max_attempts}): {message}");
                debug!("Checking signer nonce in {settle_secs}s");
                async_std::task::sleep(Duration::from_secs(settle_secs)).await;
                let block = client.at_current_block().await?;
                let current = block
                    .transactions()
                    .account_nonce(&signer.account_id())
                    .await?;
                if current > nonce {
                    warn!("Transaction included but events not read (nonce {nonce} -> {current})");
                    return Ok(None);
                }
                last_error = message;
            }
        }
    }

    Err(CrunchError::Other(format!(
        "transaction not included after {max_attempts} attempts: {last_error}"
    )))
}
