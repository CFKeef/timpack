
use std::sync::Arc;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, COOKIE, USER_AGENT};
use thiserror::Error;

use crate::{client::Creator, constants, session::Session, signature::SignatureService};

#[derive(Default)]
pub struct CreatorBuilder {
	pub user_agent: Option<String>,
	pub xbc: Option<String>,
	pub auth_id: Option<String>,
	pub two_factor: Option<String>,
	pub session: Option<String>,
	pub proxy: Option<String>,
	pub sig_svc: Arc<SignatureService>,
}

impl CreatorBuilder {
	pub fn build_headers(
		user_agent: &str,
		xbc: &str,
		auth_id: &str,
		two_factor: Option<String>,
		session: &str,
	) -> Result<HeaderMap, OnlyFansBuilderError> {
		let mut headers = HeaderMap::new();

		headers.append(
			HeaderName::from_static(constants::APP_TOKEN_HEADER),
			HeaderValue::from_static(constants::APP_TOKEN_VALUE),
		);
		headers.append(USER_AGENT, HeaderValue::from_str(user_agent)?);
		headers.append(constants::XBC_HEADER, HeaderValue::from_str(xbc)?);
		headers.append(constants::USER_ID_HEADER, HeaderValue::from_str(auth_id)?);

		let cookies = match &two_factor {
			Some(val) => format!("sess={};auth_id={};two_factor={};", &session, &auth_id, val),
			None => format!("sess={};auth_id={};", &session, &auth_id,),
		};

		headers.append(COOKIE, HeaderValue::from_str(&cookies)?);

		Ok(headers)
	}
	pub fn build(self) -> Result<Creator, OnlyFansBuilderError> {
		let user_agent = self
			.user_agent
			.ok_or(OnlyFansBuilderError::MissingUserAgent)?;
		let xbc = self.xbc.ok_or(OnlyFansBuilderError::MissingXBC)?;
		let auth_id = self.auth_id.ok_or(OnlyFansBuilderError::MissingXBC)?;
		let two_factor = self.two_factor;

		let session = self.session.ok_or(OnlyFansBuilderError::MissingSession)?;
		let proxy_str = self.proxy.ok_or(OnlyFansBuilderError::MissingProxy)?;

		let default_headers =
			Self::build_headers(&user_agent, &xbc, &auth_id, two_factor.clone(), &session)?;

		let proxy = reqwest::Proxy::http(proxy_str)?;

		let internal = reqwest::ClientBuilder::new()
			.default_headers(default_headers.clone())
			.build()?;

		let client = reqwest::ClientBuilder::new()
			.default_headers(default_headers)
			.proxy(proxy)
			.http1_only()
			.build()?;

		let session = Session::new(user_agent, xbc, auth_id, two_factor, session);

		Ok(Creator::new(session, client, internal, self.sig_svc))
	}
	pub fn user_agent(mut self, user_agent: &str) -> Self {
		self.user_agent = Some(user_agent.to_owned());
		self
	}
	pub fn auth_id(mut self, auth_id: &str) -> Self {
		self.auth_id = Some(auth_id.to_owned());
		self
	}
	pub fn xbc(mut self, xbc: &str) -> Self {
		self.xbc = Some(xbc.to_owned());
		self
	}
	pub fn two_factor(mut self, two_factor: &str) -> Self {
		self.two_factor = Some(two_factor.to_owned());
		self
	}
	pub fn session(mut self, session: &str) -> Self {
		self.session = Some(session.to_owned());
		self
	}
	pub fn proxy(mut self, proxy: &str) -> Self {
		self.proxy = Some(proxy.to_owned());
		self
	}
	pub fn sig_svc(mut self, svc: Arc<SignatureService>) -> Self {
		self.sig_svc = svc.clone();
		self
	}
}

#[derive(Error, Debug)]
pub enum OnlyFansBuilderError {
	#[error("Missing User Agent")]
	MissingUserAgent,
	#[error("Missing XBC")]
	MissingXBC,
	#[error("Missing AuthID")]
	MissingAuthID,
	#[error("Missing Session")]
	MissingSession,
	#[error("Missing Proxy")]
	MissingProxy,
	#[error(transparent)]
	HeaderName(#[from] reqwest::header::InvalidHeaderName),
	#[error(transparent)]
	HeaderValue(#[from] reqwest::header::InvalidHeaderValue),
	#[error(transparent)]
	Http(#[from] reqwest::Error),
}

#[cfg(test)]
mod tests {

	#[tokio::test]
	async fn builds_headers() {
		let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/113.0";
		let auth_id = "*";
		let xbc = "*";
		let session = "*";

		let headers = CreatorBuilder::build_headers(
			user_agent,
			xbc,
			auth_id,
			Some("*".to_string()),
			session,
		)
		.expect("Failed to build headers");

		[
			reqwest::header::USER_AGENT.as_str(),
			reqwest::header::ACCEPT.as_str(),
			reqwest::header::ACCEPT_ENCODING.as_str(),
			reqwest::header::ACCEPT_LANGUAGE.as_str(),
			constants::APP_TOKEN_HEADER,
			constants::XBC_HEADER,
			constants::USER_ID_HEADER,
		]
		.into_iter()
		.for_each(|f| {
			assert!(headers.contains_key(f), "{} wasn't set", f);
		});
	}
}
