// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use log::{error, warn};
use openidconnect::{
    AdditionalClaims, Client, ClientId, ClientSecret, EmptyExtraTokenFields, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IdTokenFields, IssuerUrl, Nonce, PkceCodeVerifier,
    StandardErrorResponse, StandardTokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
        CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreProviderMetadata,
        CoreRevocableToken, CoreRevocationErrorResponse, CoreTokenIntrospectionResponse,
        CoreTokenType,
    },
    reqwest,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct AppClaims {
    #[serde(flatten)]
    pub(crate) claims: HashMap<String, serde_json::Value>,
}
impl AdditionalClaims for AppClaims {}
type AppIdTokenFields = IdTokenFields<
    AppClaims,
    EmptyExtraTokenFields,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;

type AppTokenResponse = StandardTokenResponse<AppIdTokenFields, CoreTokenType>;
type AppErrorResponse = StandardErrorResponse<CoreErrorResponseType>;

pub(crate) type OidcClient<
    HasAuthUrl = EndpointSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointMaybeSet,
    HasUserInfoUrl = EndpointMaybeSet,
    AdditionalClaims = AppClaims,
> = Client<
    AdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    AppErrorResponse,
    AppTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
    HasUserInfoUrl,
>;

pub async fn create_oidc_client(config: &Config) -> Option<(reqwest::Client, OidcClient)> {
    let issuer_url = config.oidc_issuer_url.clone()?;
    let Some(client_id) = config.oidc_client_id.clone() else {
        warn!("OIDC Client ID must be non empty.");
        return None;
    };
    let Some(client_secret) = config.oidc_client_secret.clone() else {
        warn!("OIDC Client Secret must be non empty.");
        return None;
    };

    match IssuerUrl::new(issuer_url) {
        Ok(issuer_url) => {
            let async_http_client = reqwest::ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap();
            match CoreProviderMetadata::discover_async(issuer_url, &async_http_client).await {
                Ok(provider_metadata) => {
                    let core_client = OidcClient::from_provider_metadata(
                        provider_metadata,
                        ClientId::new(client_id),
                        Some(ClientSecret::new(client_secret)),
                    );
                    Some((async_http_client, core_client))
                }
                Err(e) => {
                    error!("Error while creating OIDC client: {:?}", e);
                    None
                }
            }
        }
        Err(e) => {
            error!("Error while processing OIDC issuer url: {:?}", e);
            None
        }
    }
}

pub(crate) struct OidcData {
    pub(crate) http_client: reqwest::Client,
    pub(crate) oidc_client: OidcClient,
    pub(crate) states: HashMap<String, (PkceCodeVerifier, Nonce)>,
}
