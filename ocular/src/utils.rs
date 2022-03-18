use http::Uri;

pub(crate) fn parse_or_build_grpc_endpoint(input: &str) -> Result<String, http::Error> {
    let mut uri = input.parse::<Uri>()?;

    if uri.port().is_none() {
        return Ok("".to_string());
    }

    if uri.scheme().is_none() {
        let builder = Uri::builder();
        uri = builder
            .scheme("https")
            // this is kind of weird
            .authority(input)
            .path_and_query("/")
            .build()?;
    }

    Ok(uri.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assay::assay;

    #[assay]
    fn valid_output() {
        let a = "test.com";
        let b = "test.com:9090";
        let c = "tcp://test.com";
        let d = "tcp://test.com:9090";
        let e = "https://test.com";
        let f = "https://test.com:9090";

        assert!(parse_or_build_grpc_endpoint(a).is_ok());
        assert_eq!(parse_or_build_grpc_endpoint(a).unwrap(), "".to_string());
        assert!(parse_or_build_grpc_endpoint(b).is_ok());
        assert_eq!(
            parse_or_build_grpc_endpoint(b).unwrap(),
            "https://test.com:9090/".to_string()
        );
        assert!(parse_or_build_grpc_endpoint(c).is_ok());
        assert_eq!(parse_or_build_grpc_endpoint(c).unwrap(), "".to_string());
        assert!(parse_or_build_grpc_endpoint(d).is_ok());
        assert_eq!(
            parse_or_build_grpc_endpoint(d).unwrap(),
            "tcp://test.com:9090/".to_string()
        );
        assert!(parse_or_build_grpc_endpoint(e).is_ok());
        assert_eq!(parse_or_build_grpc_endpoint(e).unwrap(), "".to_string());
        assert!(parse_or_build_grpc_endpoint(f).is_ok());
        assert_eq!(
            parse_or_build_grpc_endpoint(f).unwrap(),
            "https://test.com:9090/".to_string()
        );
    }
}
