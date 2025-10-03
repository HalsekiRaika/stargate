pub trait DependOnAppConfig: 'static + Sync + Send {
    type AppConfig: AppConfig;
    fn app_config(&self) -> &Self::AppConfig;
}

pub trait AppConfig {
    fn hostname(&self) -> &str;
}
