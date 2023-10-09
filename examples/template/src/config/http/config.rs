/* pub(super) fn security_headers() -> DefaultHeaders {
    use hextacy::web::xhttp::security_headers::*;
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
 */
