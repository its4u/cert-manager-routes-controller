use std::collections::HashSet;

pub fn format_cert_annotation(
    cert_annotation: Option<&String>,
    route_name: &str,
    route_namespace: &str,
) -> String {
    match cert_annotation {
        Some(cert_annotation) => {
            let mut annotations: HashSet<String> = HashSet::new();
            cert_annotation.split(",").for_each(|annotation| {
                annotations.insert(annotation.to_owned());
            });
            annotations.insert(format!("{}:{}", route_namespace, route_name));
            annotations.into_iter().collect::<Vec<String>>().join(",")
        }
        None => format!("{}:{}", route_namespace, route_name),
    }
}

pub fn format_cert_name(hostname: &str) -> String {
    format!("{}-cert", hostname)
}

pub fn format_secret_name(hostname: &str) -> String {
    format!("{}-tls", hostname)
}
