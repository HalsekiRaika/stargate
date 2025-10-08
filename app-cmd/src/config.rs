pub trait DependOnAppConfig: 'static + Sync + Send {
    fn host_name(&self) -> &str;
}
