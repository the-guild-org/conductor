use conductor_config::EndpointDefinition;

#[derive(Debug)]
pub struct EndpointRuntime {
    pub config: EndpointDefinition,
}

impl EndpointRuntime {
    #[cfg(feature = "test_utils")]
    pub fn dummy() -> Self {
        EndpointRuntime {
            config: EndpointDefinition {
                from: "dummy".to_string(),
                path: "/".to_string(),
                plugins: None,
            },
        }
    }
}
