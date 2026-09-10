use crate::substrate::CrunchConfig;
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    config::ParamsFor,
    tx::{Payload, Signer, TransactionProgress},
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
