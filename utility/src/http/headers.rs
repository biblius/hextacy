use actix_web::http::header::*;
use actix_web::middleware::DefaultHeaders;

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy
pub fn default_content_security_policy() -> (HeaderName, HeaderValue) {
    (
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(&format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            "default-src 'self';",
            "base-uri 'self';",
            "font-src 'self' https: data;",
            "form-action 'self';",
            "frame-ancestors 'self';",
            "img-src 'self' data:;",
            "object-src 'none';",
            "script-src 'self';",
            "script-src-attr 'none';",
            "style-src 'self' https: 'unsafe-inline';",
            "upgrade-insecure-requests"
        ))
        .expect("Couldn't construct CSP header"),
    )
}

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Embedder-Policy
///
/// Accepts: `"require-corp" | "unsafe-none"`
pub fn cross_origin_embedder_policy(policy: &'static str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("cross-origin-embedder-policy"),
        HeaderValue::from_static(match policy {
            "require-corp" | "unsafe-none" => policy,
            _ => panic!("Invalid value given for cross-origin-embedder-policy header"),
        }),
    )
}

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Opener-Policy
///
/// Accepts: `"same-origin" | "same-origin-allow-popups" | "unsafe-none"`
pub fn cross_origin_opener_policy(policy: &'static str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static(match policy {
            "same-origin" | "same-origin-allow-popups" | "unsafe-none" => policy,
            _ => panic!("Invalid value given for cross-origin-opener-policy header"),
        }),
    )
}

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy
///
/// Accepts: `"same-site" | "same-origin" | "cross-origin"`
pub fn cross_origin_resource_policy(policy: &'static str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static(match policy {
            "same-site" | "same-origin" | "cross-origin" => policy,
            _ => panic!("Invalid value given for cross-origin-resource-policy header"),
        }),
    )
}

/// If given a slice with 1 element the header will be set to the given element if it is valid.
/// When specifying more than 1 policy the desired policy should be given last, where every preceding
/// value is a fallback to the next if a browser doesn't support it.
/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy
///
/// Accepts: `&["no-referrer",
/// "no-referrer-when-downgrage",
/// "origin",
/// "origin-when-cross-origin",
/// "same-origin",
/// "strict-origin",
/// "strict-origin-when-cross-origin",
/// "unsafe-url"]`
pub fn referrer_policy(policies: &[&'static str]) -> (HeaderName, HeaderValue) {
    const POLICIES: &[&str] = &[
        "no-referrer",
        "no-referrer-when-downgrade",
        "origin",
        "origin-when-cross-origin",
        "same-origin",
        "strict-origin",
        "strict-origin-when-cross-origin",
        "unsafe-url",
    ];
    for p in policies {
        if !POLICIES.contains(p) {
            panic!("Invalid value given for referrer-policy header")
        }
    }
    (
        REFERRER_POLICY,
        HeaderValue::from_str(&policies.join(", ")).unwrap(),
    )
}

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security#preloading_strict_transport_security
///
/// Accepts: `max_age` in seconds, `Some("includeSubDomains" | "preload")` as an extra option or `None`
pub fn strict_transport_security(
    max_age: usize,
    option: Option<&str>,
) -> (HeaderName, HeaderValue) {
    let value = if let Some(option) = option {
        match option {
            "includeSubDomains" | "preload" => format!("max-age={}; {}", max_age, option),
            _ => panic!("Invalid value given for strict-transport-security header"),
        }
    } else {
        format!("max-age={}", max_age)
    };
    (
        STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_str(&value).unwrap(),
    )
}

/// Sets the `x-content-type-options` header to `nosniff`
/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options
pub fn no_sniff() -> (HeaderName, HeaderValue) {
    (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"))
}

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-DNS-Prefetch-Control
///
/// Accepts: `true => "on"`, `false => "off"`
pub fn dns_prefetch_control(on: bool) -> (HeaderName, HeaderValue) {
    (
        X_DNS_PREFETCH_CONTROL,
        HeaderValue::from_static(if on { "on" } else { "off" }),
    )
}

/// Specific to IE8
pub fn ie_no_open() -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("x-download-options"),
        HeaderValue::from_static("noopen"),
    )
}

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options
///
/// Accepts: `true => "sameorigin"`, `false => "deny"`
pub fn frame_options(sameorigin: bool) -> (HeaderName, HeaderValue) {
    (
        X_FRAME_OPTIONS,
        HeaderValue::from_static(if sameorigin { "sameorigin" } else { "deny" }),
    )
}

/// See https://owasp.org/www-project-secure-headers/#x-permitted-cross-domain-policies
///
/// Accepts: `"none" | "master-only" | "by-content-type" | "all"`
pub fn cross_domain_policies(policy: &'static str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("x-permitted-cross-domain-policies"),
        HeaderValue::from_static(match policy {
            "none" | "master-only" | "by-content-type" | "all" => policy,
            _ => panic!("Invalid value for x-permitted-cross-domain-policies header"),
        }),
    )
}

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection
///
/// Accepts: `true => "1; mode=block"`, `false => "0"`
pub fn xss_filter(on: bool) -> (HeaderName, HeaderValue) {
    (
        X_XSS_PROTECTION,
        HeaderValue::from_static(if on { "1; mode=block" } else { "0" }),
    )
}

/// Builds the default security header middlware
pub fn default_security_headers() -> DefaultHeaders {
    DefaultHeaders::new()
        .add(default_content_security_policy())
        .add(cross_origin_embedder_policy("require-corp"))
        .add(cross_origin_opener_policy("same-origin"))
        .add(cross_origin_resource_policy("same-origin"))
        .add(referrer_policy(&["no-referrer", "same-origin"]))
        .add(strict_transport_security(
            31536000, // 1 year
            Some("includeSubDomains"),
        ))
        .add(no_sniff())
        .add(dns_prefetch_control(false))
        .add(ie_no_open())
        .add(frame_options(true))
        .add(cross_domain_policies("none"))
        .add(xss_filter(false))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn headers() {
        let (name, value) = cross_origin_embedder_policy("require-corp");
        assert_eq!(name.to_string(), "cross-origin-embedder-policy");
        assert_eq!(value.to_str().unwrap(), "require-corp");

        let (name, value) = cross_origin_opener_policy("same-origin");
        assert_eq!(name.to_string(), "cross-origin-opener-policy");
        assert_eq!(value.to_str().unwrap(), "same-origin");

        let (name, value) = cross_origin_resource_policy("same-site");
        assert_eq!(name.to_string(), "cross-origin-resource-policy");
        assert_eq!(value.to_str().unwrap(), "same-site");

        let (name, value) = referrer_policy(&["no-referrer", "origin"]);
        assert_eq!(value.to_str().unwrap(), "no-referrer, origin");
        assert_eq!(name.to_string(), "referrer-policy");
        let (_, value) = referrer_policy(&["origin"]);
        assert_eq!(value.to_str().unwrap(), "origin");

        let (name, value) = strict_transport_security(31_536_000, Some("includeSubDomains"));
        assert_eq!(name.to_string(), "strict-transport-security");
        assert_eq!(
            value.to_str().unwrap(),
            "max-age=31536000; includeSubDomains"
        );

        let (name, value) = no_sniff();
        assert_eq!(name.to_string(), "x-content-type-options");
        assert_eq!(value.to_str().unwrap(), "nosniff");

        let (name, value) = dns_prefetch_control(false);
        assert_eq!(name.to_string(), "x-dns-prefetch-control");
        assert_eq!(value.to_str().unwrap(), "off");

        let (name, value) = ie_no_open();
        assert_eq!(name.to_string(), "x-download-options");
        assert_eq!(value.to_str().unwrap(), "noopen");

        let (name, value) = frame_options(true);
        assert_eq!(name.to_string(), "x-frame-options");
        assert_eq!(value.to_str().unwrap(), "sameorigin");

        let (name, value) = cross_domain_policies("none");
        assert_eq!(name.to_string(), "x-permitted-cross-domain-policies");
        assert_eq!(value.to_str().unwrap(), "none");

        let (name, value) = xss_filter(true);
        assert_eq!(name.to_string(), "x-xss-protection");
        assert_eq!(value.to_str().unwrap(), "1; mode=block");
    }

    #[test]
    fn _default_security_headers() {
        let _ = default_security_headers();
    }
}
