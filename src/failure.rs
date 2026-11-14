use std::{error::Error, fmt, io};

/// Formats diagnostic fields without exposing error messages, URLs or response content.
pub struct Detail<'a>(pub &'a (dyn Error + 'static));

impl fmt::Display for Detail<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(error) = self.0.downcast_ref::<ureq::Error>() {
            let kind = match error {
                ureq::Error::StatusCode(code) => {
                    return write!(f, "kind=http_status status={code}");
                }
                ureq::Error::Timeout(phase) => return write!(f, "kind=timeout phase={phase:?}"),
                ureq::Error::Io(error) => return write!(f, "transport=ureq {}", Detail(error)),
                ureq::Error::HostNotFound => "dns",
                ureq::Error::NativeTls(_) | ureq::Error::Tls(_) => "tls",
                ureq::Error::TlsRequired | ureq::Error::RequireHttpsOnly(_) => "tls_required",
                ureq::Error::ConnectionFailed => "connection_failed",
                ureq::Error::ConnectProxyFailed(_) => "proxy_connect",
                ureq::Error::InvalidProxyUrl => "proxy_config",
                ureq::Error::TooManyRedirects => "redirect_limit",
                ureq::Error::RedirectFailed => "redirect_failed",
                ureq::Error::BadUri(_) | ureq::Error::Http(_) => "invalid_request",
                ureq::Error::Protocol(_) => "http_protocol",
                ureq::Error::LargeResponseHeader(size, limit) => {
                    return write!(f, "kind=header_limit size={size} limit={limit}");
                }
                ureq::Error::BodyExceedsLimit(limit) => {
                    return write!(f, "kind=body_limit limit={limit}");
                }
                _ => "unclassified_ureq",
            };
            return write!(f, "kind={kind}");
        }
        if let Some(error) = self.0.downcast_ref::<io::Error>() {
            return write!(
                f,
                "kind=io io_kind={:?} os_code={:?}",
                error.kind(),
                error.raw_os_error()
            );
        }
        if let Some(error) = self.0.downcast_ref::<serde_json::Error>() {
            return write!(
                f,
                "kind=json json_kind={:?} line={} column={}",
                error.classify(),
                error.line(),
                error.column()
            );
        }
        if let Some(error) = self.0.downcast_ref::<DataError>() {
            return error.fmt(f);
        }
        f.write_str("kind=unclassified")
    }
}

#[derive(Debug)]
pub(crate) enum DataError {
    Missing(&'static str),
    UpstreamCode(i32),
    Empty,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(field) => write!(f, "kind=missing_data field={field}"),
            Self::UpstreamCode(code) => write!(f, "kind=upstream_code code={code}"),
            Self::Empty => f.write_str("kind=no_usable_stories"),
        }
    }
}

impl Error for DataError {}
